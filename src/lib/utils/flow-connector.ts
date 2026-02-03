// SVG flow-connector overlay for multi-row ActionBlock strips.

interface Block {
  cx: number;
  cy: number;
  top: number;
  bottom: number;
  right: number;
  left: number;
}

interface Row {
  blocks: Block[];
}

function groupByRow(blocks: Block[], tolerance = 12): Row[] {
  const rows: Row[] = [];
  for (const b of blocks) {
    const existing = rows.find((r) => Math.abs(r.blocks[0].cy - b.cy) < tolerance);
    if (existing) {
      existing.blocks.push(b);
    } else {
      rows.push({ blocks: [b] });
    }
  }
  // Sort rows top-to-bottom, blocks left-to-right within each row
  rows.sort((a, b) => a.blocks[0].cy - b.blocks[0].cy);
  for (const row of rows) {
    row.blocks.sort((a, b) => a.cx - b.cx);
  }
  return rows;
}

// Build a rounded-corner polyline: arcs at each bend with radius r.
function roundedPolyline(pts: [number, number][], r: number): string {
  if (pts.length < 2) return "";
  if (pts.length === 2) return `M ${pts[0][0]},${pts[0][1]} L ${pts[1][0]},${pts[1][1]}`;

  const d: string[] = [];
  d.push(`M ${pts[0][0]},${pts[0][1]}`);

  for (let i = 1; i < pts.length - 1; i++) {
    const prev = pts[i - 1];
    const cur = pts[i];
    const next = pts[i + 1];

    // Vectors from corner to adjacent points
    const dx1 = prev[0] - cur[0], dy1 = prev[1] - cur[1];
    const dx2 = next[0] - cur[0], dy2 = next[1] - cur[1];
    const len1 = Math.hypot(dx1, dy1);
    const len2 = Math.hypot(dx2, dy2);

    // Clamp radius so it doesn't exceed half the segment length
    const clamp = Math.min(r, len1 / 2, len2 / 2);

    // Points where the arc starts/ends
    const ax = cur[0] + (dx1 / len1) * clamp;
    const ay = cur[1] + (dy1 / len1) * clamp;
    const bx = cur[0] + (dx2 / len2) * clamp;
    const by = cur[1] + (dy2 / len2) * clamp;

    d.push(`L ${ax},${ay}`);
    d.push(`Q ${cur[0]},${cur[1]} ${bx},${by}`);
  }

  const last = pts[pts.length - 1];
  d.push(`L ${last[0]},${last[1]}`);
  return d.join(" ");
}

function buildPath(container: HTMLElement): string {
  const children = container.querySelectorAll<HTMLElement>("[data-flow-block]");
  if (children.length < 2) return "";

  const containerRect = container.getBoundingClientRect();
  const style = getComputedStyle(container);
  const padLeft = parseFloat(style.paddingLeft) || 0;
  const padRight = parseFloat(style.paddingRight) || 0;

  const blocks: Block[] = [];

  for (const el of children) {
    const r = el.getBoundingClientRect();
    blocks.push({
      cx: r.left + r.width / 2 - containerRect.left,
      cy: r.top + r.height / 2 - containerRect.top,
      top: r.top - containerRect.top,
      bottom: r.bottom - containerRect.top,
      right: r.right - containerRect.left,
      left: r.left - containerRect.left,
    });
  }

  const rows = groupByRow(blocks);
  const parts: string[] = [];
  const cornerR = 12;

  for (let ri = 0; ri < rows.length; ri++) {
    const row = rows[ri];

    // Horizontal connectors within the row
    for (let bi = 0; bi < row.blocks.length - 1; bi++) {
      const a = row.blocks[bi];
      const b = row.blocks[bi + 1];
      const y = (a.cy + b.cy) / 2;
      parts.push(`M ${a.right},${y} L ${b.left},${y}`);
    }

    // Stepped connector between rows routed through padding zones
    if (ri < rows.length - 1) {
      const nextRow = rows[ri + 1];
      const lastBlock = row.blocks[row.blocks.length - 1];
      const nextFirst = nextRow.blocks[0];

      const x1 = lastBlock.right;
      const y1 = lastBlock.cy;
      const x2 = nextFirst.left;
      const y2 = nextFirst.cy;

      const rowBottom = Math.max(...row.blocks.map((b) => b.bottom));
      const nextRowTop = Math.min(...nextRow.blocks.map((b) => b.top));
      const gapY = (rowBottom + nextRowTop) / 2;

      // Right turn just past the last block; left turn in the padding zone
      const turnR = x1 + cornerR + 4;
      const turnL = padLeft / 2;

      parts.push(roundedPolyline([
        [x1, y1], [turnR, y1], [turnR, gapY], [turnL, gapY], [turnL, y2], [x2, y2],
      ], cornerR));
    }
  }

  return parts.join(" ");
}

function render(container: HTMLElement, svg: SVGSVGElement) {
  svg.setAttribute("width", String(container.offsetWidth));
  svg.setAttribute("height", String(container.offsetHeight));
  const d = buildPath(container);
  let path = svg.querySelector("path");
  if (!path) {
    path = document.createElementNS("http://www.w3.org/2000/svg", "path");
    path.setAttribute("stroke", "var(--color-border)");
    path.setAttribute("stroke-width", "3");
    path.setAttribute("stroke-linecap", "butt");
    path.setAttribute("stroke-linejoin", "round");
    path.setAttribute("fill", "none");
    svg.appendChild(path);
  }
  path.setAttribute("d", d);
}

export function flowConnector(container: HTMLElement) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.style.position = "absolute";
  svg.style.inset = "0";
  svg.style.pointerEvents = "none";
  svg.style.overflow = "visible";
  container.appendChild(svg);

  const update = () => render(container, svg);

  // Initial draw after layout settles
  requestAnimationFrame(update);

  const ro = new ResizeObserver(update);
  ro.observe(container);

  const mo = new MutationObserver(update);
  mo.observe(container, { childList: true, subtree: true });

  return {
    destroy() {
      ro.disconnect();
      mo.disconnect();
      svg.remove();
    },
  };
}
