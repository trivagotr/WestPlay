/** Theme metadata is colors only: no wallpaper URL, private path, or video copy. */
export function applyShellStyle(root, style) {
  if (
    !style ||
    !["dark", "light"].includes(style.appearance) ||
    !Array.isArray(style.palette) ||
    style.palette.length !== 3 ||
    !style.palette.every(
      (c) => typeof c === "string" && /^#[a-f0-9]{6}$/i.test(c),
    )
  )
    return false;
  root.dataset.appearance = style.appearance;
  ["left", "center", "right"].forEach((part, i) =>
    root.style.setProperty("--cabin-wallpaper-" + part, style.palette[i]),
  );
  return true;
}
