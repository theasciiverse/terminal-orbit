use crate::constants::{CONSAGE, CONSLINES};
use crate::types::World;

/// Add a message to the scrolling console (port of Cprint from console.c).
pub fn console_add(world: &mut World, msg: impl Into<String>) {
    let c = &mut world.console;
    if c.next >= CONSLINES {
        for i in 0..CONSLINES - 1 {
            c.buf[i] = std::mem::take(&mut c.buf[i + 1]);
            c.age[i] = c.age[i + 1];
        }
        c.next = CONSLINES - 1;
    }
    c.buf[c.next] = msg.into();
    c.age[c.next] = 0.0;
    c.next += 1;
}

/// Advance ages of all buffered console messages by `dt` seconds.
pub fn advance_console(world: &mut World, dt: f64) {
    let n = world.console.next;
    for i in 0..n {
        world.console.age[i] += dt;
    }
}

/// Return the subset of console lines still young enough to display.
pub fn active_lines(world: &World) -> Vec<&str> {
    (0..world.console.next)
        .filter(|&i| world.console.age[i] < CONSAGE)
        .map(|i| world.console.buf[i].as_str())
        .collect()
}
