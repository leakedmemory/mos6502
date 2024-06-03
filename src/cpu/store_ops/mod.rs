pub(super) mod sta;
pub(super) mod stx;
pub(super) mod sty;

pub(super) use sta::sta_zero_page;
pub(super) use sta::sta_zero_page_x;
pub(super) use sta::sta_absolute;
pub(super) use sta::sta_absolute_x;
pub(super) use sta::sta_absolute_y;
pub(super) use sta::sta_indirect_x;
pub(super) use sta::sta_indirect_y;

pub(super) use stx::stx_zero_page;
pub(super) use stx::stx_zero_page_y;
pub(super) use stx::stx_absolute;

pub(super) use sty::sty_zero_page;
pub(super) use sty::sty_zero_page_x;
pub(super) use sty::sty_absolute;
