/**
 * Background script for Ripple extension
 * Intercepts web requests and cleans tracking parameters
 */

import init, { clean_url, default_options, CleaningOptions, CleaningResult } from '../../pkg/plink.js';

// State management
let isInitialized = false;
let cleaningOptions: CleaningOptions;
let globalEnabled = true;
let statistics = {
  totalCleaned: 0,
  totalBlocked: 0,
  sessionStart: Date.now()
};

/**
 * Initialize the WASM module and load settings
 */
async function initialize() {
  try {
    // Initialize WASM module
    await init();
    
    // Load default options
    cleaningOptions = default_options();
    
    // Load saved settings from storage
    const stored = await browser.storage.local.get([
      'globalEnabled',
      'cleaningOptions',
      'statistics'
    ]);
    
    if (stored.globalEnabled !== undefined) {
      globalEnabled = stored.globalEnabled;
    }
    
    if (stored.cleaningOptions) {
      cleaningOptions = { ...cleaningOptions, ...stored.cleaningOptions };
    }
    
    if (stored.statistics) {
      statistics = stored.statistics;
    }
    
    isInitialized = true;
    console.log('[Ripple] Extension initialized successfully');
    updateBadge();
  } catch (error) {
    console.error('[Ripple] Initialization failed:', error);
    isInitialized = false;
  }
}

/**
 * Update extension badge with cleaning count
 */
function updateBadge() {
  const count = statistics.totalCleaned.toString();
  browser.action.setBadgeText({ text: count });
  browser.action.setBadgeBackgroundColor({ color: '#4CAF50' });
}

/**
 * Check if URL is a data URL
 */
function isDataURL(url: string): boolean {
  return url.startsWith('data:');
}

/**
 * Check if URL is a localhost/local network URL
 */
function isLocalURL(url: string): boolean {
  try {
    const urlObj = new URL(url);
    const hostname = urlObj.hostname.toLowerCase();
    
    return (
      hostname === 'localhost' ||
      hostname === '127.0.0.1' ||
      hostname === '::1' ||
      hostname.endsWith('.local') ||
      /^192\.168\.\d{1,3}\.\d{1,3}$/.test(hostname) ||
      /^10\.\d{1,3}\.\d{1,3}\.\d{1,3}$/.test(hostname) ||
      /^172\.(1[6-9]|2\d|3[0-1])\.\d{1,3}\.\d{1,3}$/.test(hostname)
    );
  } catch {
    return false;
  }
}

/**
 * Main request handler - intercepts and cleans URLs
 */
function handleRequest(
  details: browser.webRequest._OnBeforeRequestDetails
): browser.webRequest.BlockingResponse | void {
  // Skip if not initialized or disabled
  if (!isInitialized || !globalEnabled) {
    return {};
  }
  
  // Skip data URLs
  if (isDataURL(details.url)) {
    return {};
  }
  
  // Skip localhost if configured
  if (cleaningOptions.skip_localhost && isLocalURL(details.url)) {
    return {};
  }
  
  try {
    // Clean the URL using WASM module
    const result: CleaningResult = clean_url(details.url, cleaningOptions);
    
    // Handle cancellation (domain blocking)
    if (result.cancel) {
      statistics.totalBlocked++;
      updateBadge();
      saveStatistics();
      
      // For main frame requests, redirect to blocked page
      if (details.type === 'main_frame') {
        const blockedPage = browser.runtime.getURL(
          `blocked.html?url=${encodeURIComponent(details.url)}`
        );
        browser.tabs.update(details.tabId!, { url: blockedPage });
        return { cancel: true };
      }
      
      return { cancel: true };
    }
    
    // Handle redirect or URL changes
    if (result.changed || result.redirect) {
      statistics.totalCleaned++;
      updateBadge();
      saveStatistics();
      
      // Log the cleaning action
      logCleaning(details.url, result.url, result.applied_rules);
      
      return { redirectUrl: result.url };
    }
    
    // No changes needed
    return {};
  } catch (error) {
    console.error('[Ripple] Error cleaning URL:', error);
    return {};
  }
}

/**
 * Log cleaning action
 */
function logCleaning(originalUrl: string, cleanedUrl: string, rules: string[]) {
  const logEntry = {
    timestamp: Date.now(),
    original: originalUrl,
    cleaned: cleanedUrl,
    rules: rules,
    saved: originalUrl.length - cleanedUrl.length
  };
  
  // Store in memory (could be persisted to storage if needed)
  browser.storage.local.get('cleaningLog').then((data) => {
    const log = data.cleaningLog || [];
    log.unshift(logEntry);
    
    // Keep only last 1000 entries
    if (log.length > 1000) {
      log.length = 1000;
    }
    
    browser.storage.local.set({ cleaningLog: log });
  });
}

/**
 * Save statistics to storage
 */
let saveTimeout: number | null = null;
function saveStatistics() {
  // Debounce saves to avoid excessive writes
  if (saveTimeout) {
    clearTimeout(saveTimeout);
  }
  
  saveTimeout = setTimeout(() => {
    browser.storage.local.set({ statistics });
    saveTimeout = null;
  }, 1000);
}

enum MessageType {
  GetStatistics, 
  GetSettings, 
  ToggleEnabled,
  UpdateOptions,
  ResetStatistics,
  CleanUrl,
}

/**
 * Handle messages from popup/content scripts
 */
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  switch (message.type) {
    case MessageType.GetStatistics:
      sendResponse(statistics);
      break;

    case MessageType.GetSettings:
      sendResponse({
        globalEnabled,
        cleaningOptions,
      });
      break;

    case MessageType.ToggleEnabled:
      globalEnabled = !globalEnabled;
      browser.storage.local.set({ globalEnabled });
      updateBadge();
      sendResponse({ enabled: globalEnabled });
      break;

    case MessageType.UpdateOptions:
      cleaningOptions = { ...cleaningOptions, ...message.options };
      browser.storage.local.set({ cleaningOptions });
      sendResponse({ success: true });
      break;

    case MessageType.ResetStatistics:
      statistics = {
        totalCleaned: 0,
        totalBlocked: 0,
        sessionStart: Date.now(),
      };
      saveStatistics();
      updateBadge();
      sendResponse({ success: true });
      break;

    case MessageType.CleanUrl:
      // Manual URL cleaning from popup
      try {
        const result = clean_url(message.url, cleaningOptions);
        sendResponse(result);
      } catch (error) {
        sendResponse({ error: String(error) });
      }
      break;
  }

  return true; // Keep channel open for async response
});

/**
 * Register webRequest listener
 */
browser.webRequest.onBeforeRequest.addListener(
  handleRequest,
  {
    urls: ['<all_urls>'],
    types: [
      'main_frame',
      'sub_frame',
      'stylesheet',
      'script',
      'image',
      'font',
      'object',
      'xmlhttprequest',
      'ping',
      'csp_report',
      'media',
      'websocket',
      'other'
    ]
  },
  ['blocking']
);

// Initialize on startup
initialize();

// Handle extension updates
browser.runtime.onInstalled.addListener((details) => {
  if (details.reason === 'install') {
    console.log('[Ripple] Extension installed');
    // Open welcome page
    browser.tabs.create({
      url: browser.runtime.getURL('welcome.html')
    });
  } else if (details.reason === 'update') {
    console.log('[Ripple] Extension updated');
  }
});

// Export for testing
export { handleRequest, initialize, isLocalURL };
