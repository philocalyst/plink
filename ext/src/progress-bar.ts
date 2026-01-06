import svgBuilder, { SVGBuilderInstance } from 'svg-builder';

interface ProgressBarOptions {
  greenWidth: number;
  purpleWidth: number;
  yellowWidth: number;
  height?: number;
}

const styleContent = `
  @keyframes slide {
    0% { transform: translateX(0); }
    100% { transform: translateX(20px); }
  }
  .anim-group {
    animation: slide 5s linear infinite;
  }
`;

export function createProgressBar({
  greenWidth,
  purpleWidth,
  yellowWidth,
  height = 60,
}: ProgressBarOptions): string {
  const totalWidth = greenWidth + purpleWidth + yellowWidth;
  const radius = height / 2;

  // This creates the background color and the transparent sliding stripes
  const createPatternContent = (color: string) => {
    return svgBuilder
      .create()
      .rect({ width: 20, height, fill: color })
      .g(
        { class: 'anim-group' },
        svgBuilder
          .create()
          .rect({ width: 10, height, fill: 'rgba(0,0,0,0.2)', x: 0 })
          .rect({ width: 10, height, fill: 'rgba(0,0,0,0.2)', x: -20 })
      );
  };

  const defs = svgBuilder
    .create()
    // Green Pattern
    .pattern(
      {
        id: 'stripe-green',
        patternUnits: 'userSpaceOnUse',
        width: 20,
        height,
        patternTransform: 'rotate(45)',
      },
      createPatternContent('#22c55e')
    )
    // Purple Pattern
    .pattern(
      {
        id: 'stripe-purple',
        patternUnits: 'userSpaceOnUse',
        width: 20,
        height,
        patternTransform: 'rotate(45)',
      },
      createPatternContent('#a855f7')
    )
    // Yellow Pattern
    .pattern(
      {
        id: 'stripe-yellow',
        patternUnits: 'userSpaceOnUse',
        width: 20,
        height,
        patternTransform: 'rotate(45)',
      },
      createPatternContent('#eab308')
    )
    // Clip Path (The pill shape)
    .clipPath(
      { id: 'pill-clip' },
      svgBuilder
        .create()
        .rect({ x: 0, y: 0, width: totalWidth, height, rx: radius, ry: radius })
    );

  // x-coordinates are calculated cumulatively
  const segments = svgBuilder
    .create()
    .rect({
      x: 0,
      y: 0,
      width: greenWidth,
      height,
      fill: 'url(#stripe-green)',
    })
    .rect({
      x: greenWidth,
      y: 0,
      width: purpleWidth,
      height,
      fill: 'url(#stripe-purple)',
    })
    .rect({
      x: greenWidth + purpleWidth,
      y: 0,
      width: yellowWidth,
      height,
      fill: 'url(#stripe-yellow)',
    });

  // Final assembly
  return svgBuilder
    .create()
    .width('100%') 
    .height(height)
    .viewBox(`0 0 ${totalWidth} ${height}`)
    .style({}, styleContent)
    .defs(undefined, defs)
    // Background background pill (dark gray)
    .rect({
      x: 0,
      y: 0,
      width: totalWidth,
      height,
      rx: radius,
      ry: radius,
      fill: '#333',
    })
    // Colored segments (Clipped)
    .g({ 'clip-path': 'url(#pill-clip)' }, segments)
    // Border overlay
    .rect({
      x: 0,
      y: 0,
      width: totalWidth,
      height,
      rx: radius,
      ry: radius,
      fill: 'none',
      stroke: '#555',
      'stroke-width': 2,
    })
    .render();
}
