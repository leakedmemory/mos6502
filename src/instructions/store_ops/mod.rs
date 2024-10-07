pub(crate) mod sta;
pub(crate) mod stx;
pub(crate) mod sty;

pub(crate) use sta::{
    sta_absolute, sta_absolute_x, sta_absolute_y, sta_indirect_x, sta_indirect_y, sta_zero_page,
    sta_zero_page_x,
};
pub(crate) use stx::{stx_absolute, stx_zero_page, stx_zero_page_y};
pub(crate) use sty::{sty_absolute, sty_zero_page, sty_zero_page_x};
