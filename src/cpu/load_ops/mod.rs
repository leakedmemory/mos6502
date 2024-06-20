pub(super) mod lda;
pub(super) mod ldx;
pub(super) mod ldy;

pub(super) use lda::{
    lda_absolute, lda_absolute_x, lda_absolute_y, lda_immediate, lda_indirect_x, lda_indirect_y,
    lda_zero_page, lda_zero_page_x,
};

pub(super) use ldx::{ldx_absolute, ldx_absolute_y, ldx_immediate, ldx_zero_page, ldx_zero_page_y};

pub(super) use ldy::{ldy_absolute, ldy_absolute_x, ldy_immediate, ldy_zero_page, ldy_zero_page_x};
