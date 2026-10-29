-- Keycorner: a movable keyboard reference, independent of the tiling layout.
o.window({ title = "^Keycorner Overlay$" }, {
  float = true,
  pin = true,
  no_initial_focus = true,
  no_follow_mouse = true,
  decorate = false,
  tag = "-default-opacity",
  opacity = "1 1",
  move = { "18", "monitor_h-window_h-18" },
})
o.window({ title = "^Keycorner Settings$" }, {
  float = true,
  center = true,
  size = { 510, 700 },
})
