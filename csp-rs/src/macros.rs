/* Manual Implementations of required macros as cbindgen does not generate these correctly (GH issue #753) */
#![no_std]
#![allow(unused)]

// GETs
macro_rules! cfp_field {
    ($id:expr, $rsiz:expr, $fsiz:expr) => {
        (($id >> $rsiz) & ((1u32 << $fsiz) - 1))
    };
}

macro_rules! cfp_type {
    ($id:expr) => {
        cfp_field!(
            $id,
            crate::libcsp_ffi::CFP_REMAIN_SIZE + crate::libcsp_ffi::CFP_ID_SIZE,
            crate::libcsp_ffi::CFP_TYPE_SIZE
        )
    };
}

macro_rules! cfp_remain {
    ($id:expr) => {
        cfp_field!(
            $id,
            crate::libcsp_ffi::CFP_ID_SIZE,
            crate::libcsp_ffi::CFP_REMAIN_SIZE
        )
    };
}

// MAKEs
macro_rules! cfp_make_field {
    ($id:expr, $fsiz:expr, $rsiz:expr) => {
        (((($id) & ((1u32 << ($fsiz)) - 1)) as u32) << ($rsiz))
    };
}

macro_rules! cfp_make_src {
    ($id:expr) => {
        cfp_make_field!(
            $id,
            crate::libcsp_ffi::CFP_HOST_SIZE,
            crate::libcsp_ffi::CFP_HOST_SIZE
                + crate::libcsp_ffi::CFP_TYPE_SIZE
                + crate::libcsp_ffi::CFP_REMAIN_SIZE
                + crate::libcsp_ffi::CFP_ID_SIZE
        )
    };
}

macro_rules! cfp_make_dst {
    ($id:expr) => {
        cfp_make_field!(
            $id,
            crate::libcsp_ffi::CFP_HOST_SIZE,
            crate::libcsp_ffi::CFP_TYPE_SIZE
                + crate::libcsp_ffi::CFP_REMAIN_SIZE
                + crate::libcsp_ffi::CFP_ID_SIZE
        )
    };
}

macro_rules! cfp_make_type {
    ($id:expr) => {
        cfp_make_field!(
            $id,
            crate::libcsp_ffi::CFP_TYPE_SIZE,
            crate::libcsp_ffi::CFP_REMAIN_SIZE + crate::libcsp_ffi::CFP_ID_SIZE
        )
    };
}

macro_rules! cfp_make_remain {
    ($id:expr) => {
        cfp_make_field!(
            $id,
            crate::libcsp_ffi::CFP_REMAIN_SIZE,
            crate::libcsp_ffi::CFP_ID_SIZE
        )
    };
}

macro_rules! cfp_make_id {
    ($id:expr) => {
        cfp_make_field!($id, crate::libcsp_ffi::CFP_ID_SIZE, 0)
    };
}
