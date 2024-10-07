pub(crate) mod lda;
pub(crate) mod ldx;
pub(crate) mod ldy;

pub(crate) use lda::{
    lda_absolute, lda_absolute_x, lda_absolute_y, lda_immediate, lda_indirect_x, lda_indirect_y,
    lda_zero_page, lda_zero_page_x,
};

pub(crate) use ldx::{ldx_absolute, ldx_absolute_y, ldx_immediate, ldx_zero_page, ldx_zero_page_y};

pub(crate) use ldy::{ldy_absolute, ldy_absolute_x, ldy_immediate, ldy_zero_page, ldy_zero_page_x};
