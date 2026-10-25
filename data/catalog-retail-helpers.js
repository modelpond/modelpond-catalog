// Icon order is fixed: audio, then image, then text.
function catalogModalities(list) {
  const mods = CATALOG_ORDER.filter((m) => (list || []).indexOf(m) !== -1);
  if (!mods.length) return '<span class="mod-none">-</span>';
  return mods.map((m) =>
    '<span class="mod" title="' + m + '">' + CATALOG_ICONS[m] +
    '<span class="sr-only">' + m + '</span></span>'
  ).join("");
}

function catalogPrice(entry) {
  if (!entry || !entry.unit || !entry.value) return "-";
  const dp = entry.dp == null ? 2 : entry.dp;
  return entry.value.toFixed(dp) + '<span class="unit">/' + entry.unit + '</span>';
}

function catalogPriceText(entry) {
  if (!entry || !entry.unit || !entry.value) return "-";
  const dp = entry.dp == null ? 2 : entry.dp;
  return entry.value.toFixed(dp) + "/" + entry.unit;
}

function catalogCompare(a, b) {
  const key = (m) => (m.out && m.out.value ? m.out.value : Infinity);
  const ka = key(a), kb = key(b);
  if (ka !== kb) return ka - kb;
  const inKey = (m) => (m.in && m.in.value ? m.in.value : Infinity);
  if (inKey(a) !== inKey(b)) return inKey(a) - inKey(b);
  return a.model < b.model ? -1 : 1;
}

function catalogRow(item) {
  const cached = item.cachedIn
    ? '<small>cached ' + catalogPrice(item.cachedIn) + '</small>'
    : "";
  const outNote = item.outNote ? '<small>' + item.outNote + '</small>' : "";
  return '<div' + (item.rowClass ? ' class="' + item.rowClass + '"' : '') + '>' +
    '<span class="mt-name">' + item.model + '</span>' +
    '<span class="mt-mod">' + catalogModalities(item.input_modalities) + '</span>' +
    '<span class="mt-mod">' + catalogModalities(item.output_modalities) + '</span>' +
    '<span class="mt-in"><strong>' + catalogPrice(item.in) + '</strong>' + cached + '</span>' +
    '<span class="mt-stack"><strong>' + catalogPrice(item.out) + '</strong>' + outNote + '</span>' +
    '</div>';
}

function catalogBase(id) {
  return BASE_MODELS.find((m) => m.id === id) || null;
}
