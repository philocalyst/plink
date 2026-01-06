import svgBuilder from 'svg-builder';

interface ProgressBarOptions {
  green: number;
  purple: number;
  yellow: number;
}

export function createProgressBar({ green, purple, yellow }: ProgressBarOptions): string {
  const total = green + purple + yellow;
  
  // Convert inputs to ratios out of 100 (the viewBox width)
  const gW = (green / total) * 100;
  const pW = (purple / total) * 100;
  const yW = (yellow / total) * 100;

  const height = 60;
  const vWidth = 100; // Fixed coordinate system for the viewBox
  const radius = height / 2;

  const styleContent = `
    @keyframes slide { 0% { transform: translateX(0); } 100% { transform: translateX(20px); } }
    .anim-group { animation: slide 5s linear infinite; }
  `;

  const createPattern = (id: string, color: string) => 
    svgBuilder.create().pattern(
      { id, patternUnits: 'userSpaceOnUse', width: 20, height, patternTransform: 'rotate(45)' },
      svgBuilder.create().rect({ width: 20, height, fill: color })
        .g({ class: 'anim-group' }, 
          svgBuilder.create()
            .rect({ width: 10, height, fill: 'rgba(0,0,0,0.2)', x: 0 })
            .rect({ width: 10, height, fill: 'rgba(0,0,0,0.2)', x: -20 })
        )
    );

  const defs = svgBuilder.create()
    .defs(undefined, svgBuilder.create()
      .circle({ r: 0 }) // Dummy for defs chaining if needed
      .linearGradient(null, createPattern('stripe-green', '#22c55e')) // svg-builder quirk: patterns in defs
      .linearGradient(null, createPattern('stripe-purple', '#a855f7'))
      .linearGradient(null, createPattern('stripe-yellow', '#eab308'))
      .clipPath({ id: 'pill-clip' }, 
        svgBuilder.create().rect({ x: 0, y: 0, width: vWidth, height, rx: radius, ry: radius })
      )
    );

  return svgBuilder.create()
    .width('100%') 
    .height('auto') // Let it scale vertically based on container width
    .viewBox(`0 0 ${vWidth} ${height}`)
    .style({}, styleContent)
    .defs(undefined, defs)
    // Background
    .rect({ width: vWidth, height, rx: radius, ry: radius, fill: '#333' })
    // Segments based on Ratios
    .g({ 'clip-path': 'url(#pill-clip)' },
      svgBuilder.create()
        .rect({ x: 0, y: 0, width: gW, height, fill: 'url(#stripe-green)' })
        .rect({ x: gW, y: 0, width: pW, height, fill: 'url(#stripe-purple)' })
        .rect({ x: gW + pW, y: 0, width: yW, height, fill: 'url(#stripe-yellow)' })
    )
    .rect({ width: vWidth, height, rx: radius, ry: radius, fill: 'none', stroke: '#555', 'stroke-width': 1 })
    .render();
}
