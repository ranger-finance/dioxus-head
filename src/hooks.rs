// source: https://discord.com/channels/899851952891002890/1229857956724740218

use dioxus::{dioxus_core::use_hook, hooks::use_effect, prelude::use_drop};
use std::{cell::RefCell, rc::Rc};

pub trait UseEffectCleanup {
    fn call(self);
}

impl UseEffectCleanup for () {
    fn call(self) {}
}

impl<T: FnOnce()> UseEffectCleanup for T {
    fn call(self) {
        self()
    }
}

pub fn use_effect_with_cleanup<Cleanup: UseEffectCleanup + 'static>(
    mut callback: impl FnMut() -> Cleanup + 'static,
) {
    let cleanup = use_hook(|| Rc::new(RefCell::new(None::<Cleanup>)));

    use_effect({
        let cleanup = cleanup.clone();
        move || {
            let _ = cleanup.take().map(UseEffectCleanup::call);
            *cleanup.borrow_mut() = Some(callback());
        }
    });

    use_drop(move || {
        let _ = cleanup.take().map(UseEffectCleanup::call);
    });
}
