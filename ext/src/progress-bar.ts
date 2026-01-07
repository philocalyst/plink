import { SVG } from '@svgdotjs/svg.js';

interface ProgressBarOptions {
  green: number;
  purple: number;
  yellow: number;
}

export function createProgressBar({
  green,
  purple,
  yellow,
}: ProgressBarOptions): string {
  const total = green + purple + yellow;

  // Viewbox is 100 units wide. 
  // We calculate the width of each segment relative to that 100.
  const gW = (green / total) * 100;
  const pW = (purple / total) * 100;
  const yW = (yellow / total) * 100;

  const height = 30;
  const vWidth = '100%';
  const radius = height / 2;

  const draw = SVG();

  draw.attr('width', '100%');
  draw.attr('height', height);
  draw.attr('preserveAspectRatio', 'none');

  draw.style().addText(`
    @keyframes slide {
      0% { transform: translateX(0); }
      100% { transform: translateX(20px); }
    }
    .anim-group {
      animation: slide 5s linear infinite;
    }
  `);

  const createPattern = (id: string, color: string) => {
    const pattern = draw.pattern(20, height, (add) => {
      add.rect(20, height).fill(color);
      const animGroup = add.group().addClass('anim-group');
      animGroup.rect(10, height).fill('rgba(0,0,0,0.2)').attr('x', 0);
      animGroup.rect(10, height).fill('rgba(0,0,0,0.2)').attr('x', -20);
    });

    pattern.attr({
      id: id,
      patternUnits: 'userSpaceOnUse',
      patternTransform: 'rotate(45)',
    });

    return pattern;
  };

  createPattern('stripe-green', '#22c55e');
  createPattern('stripe-purple', '#a855f7');
  createPattern('stripe-yellow', '#eab308');

  const clip = draw.clip().attr('id', 'pill-clip');
  clip.rect(vWidth, height).attr({
    x: 0,
    y: 0,
    rx: radius,
    ry: radius,
  });

  // Background
  draw.rect(vWidth, height).attr({
    x: 0,
    y: 0,
    rx: radius,
    ry: radius,
    fill: '#333',
  });

  const segmentsGroup = draw.group().attr('clip-path', 'url(#pill-clip)');

  // Green segment - Starts at 0
  segmentsGroup.rect(gW + '%', height).attr({
    x: 0,
    y: 0,
    fill: 'url(#stripe-green)',
  });

  // Purple segment - Starts where Green ends
  segmentsGroup.rect(pW + '%', height).attr({
    x: gW,
    y: 0,
    fill: 'url(#stripe-purple)',
  });

  // Yellow segment - Starts where Purple ends
  segmentsGroup.rect(yW + '%', height).attr({
    x: gW + pW,
    y: 0,
    fill: 'url(#stripe-yellow)',
  });

  // Border
  draw.rect(vWidth, height).attr({
    x: 0,
    y: 0,
    rx: radius,
    ry: radius,
    fill: 'none',
    stroke: '#555',
    'stroke-width': 2,
  });

  return draw.svg();
}
