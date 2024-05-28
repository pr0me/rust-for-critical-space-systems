#![no_std]

pub mod csp_structs;
pub mod libcsp_ffi;
#[macro_use]
pub mod macros;

use libcsp_ffi::{csp_packet_t, csp_id_setup_rx, CFP2_DST_OFFSET};

use crate::csp_structs::*;
use core::{ffi, mem, ptr};

/// Max number of bytes per CAN frame
pub const CAN_FRAME_SIZE: u8 = 8;

/// CFP 1.x defines
pub const CFP1_CSP_HEADER_OFFSET: u8 = 0;
pub const CFP1_CSP_HEADER_SIZE: u8 = 4;
pub const CFP1_DATA_LEN_OFFSET: u8 = 4;
pub const CFP1_DATA_LEN_SIZE: u8 = 2;
pub const CFP1_DATA_OFFSET: u8 = 6;
pub const CFP1_DATA_SIZE_BEGIN: u8 = 2;
pub const CFP1_DATA_SIZE_MORE: u8 = 8;

/// Mask to uniquely separate connections
pub const CFP_ID_CONN_MASK: u32 = cfp_make_src!(((1u32 << libcsp_ffi::CFP_HOST_SIZE) - 1))
    | cfp_make_dst!(((1u32 << libcsp_ffi::CFP_HOST_SIZE) - 1))
    | cfp_make_id!(((1u32 << libcsp_ffi::CFP_ID_SIZE) - 1));

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo<'_>) -> ! { loop {} }

/// Handle incoming CSP v1.0 CAN frames
#[no_mangle]
pub extern "C" fn csp_can1_rx(
    iface: *mut libcsp_ffi::csp_iface_t,
    id: u32,
    data: *const u8,
    data_length: u8,
    task_woken: *mut i32,
) -> CspError {
    let curr_iface = unsafe {
        if !iface.is_null() {
            &mut *iface
        } else {
            return CspError::Inval;
        }
    };
    let if_data = curr_iface.interface_data as *mut libcsp_ffi::csp_can_interface_data_t;

    let curr_frame_type = {
        let type_from_raw_int = CfpFrameType::try_from(cfp_type!(id));
        if let Ok(tmp_type) = type_from_raw_int {
            tmp_type
        } else {
            // CSP_DBG_CAN_ERR_UNKNOWN
            return CspError::Inval;
        }
    };

    // bind incoming frame to a packet buffer
    let packet: &mut CspPacket = unsafe {
        let mut packet_ptr: *mut CspPacket =
            libcsp_ffi::csp_can_pbuf_find(if_data, id, CFP_ID_CONN_MASK, task_woken)
                as *mut CspPacket;
        if packet_ptr.is_null() {
            if curr_frame_type == CfpFrameType::CfpBegin {
                packet_ptr =
                    libcsp_ffi::csp_can_pbuf_new(if_data, id, task_woken) as *mut CspPacket;
                if packet_ptr.is_null() {
                    curr_iface.rx_error += 1;
                    return CspError::Nomem;
                } else {
                    &mut *packet_ptr
                }
            } else {
                curr_iface.frame += 1;
                return CspError::Inval;
            }
        } else {
            &mut *packet_ptr
        }
    };

    // reset frame data offset
    let mut offset = 0_u8;
    let packet_ptr = ptr::addr_of_mut!(*packet);

    // handle frame types
    loop {
        let mut fallthrough = false;

        if curr_frame_type == CfpFrameType::CfpBegin {
            // discard packet if DLC is less than CSP id + CSP length fields
            if (data_length as usize) < (mem::size_of::<ffi::c_uint>() + mem::size_of::<u16>()) {
                curr_iface.frame += 1;
                unsafe {
                    libcsp_ffi::csp_can_pbuf_free(
                        if_data,
                        packet_ptr as *mut csp_packet_t,
                        1,
                        task_woken,
                    );
                }
                break;
            }

            curr_iface.frame += 1;

            unsafe {
                libcsp_ffi::csp_id_setup_rx(packet_ptr as *mut csp_packet_t);

                // copy CSP identifier (header)
                let header: u32 = ptr::read_unaligned(data as *const u32);
                ptr::write_unaligned(packet.layer.rx_tx_data.frame_begin as *mut u32, header);
                packet.layer.rx_tx_data.frame_length += mem::size_of::<ffi::c_uint>() as u16;

                libcsp_ffi::csp_id_strip(packet_ptr as *mut csp_packet_t);

                // copy CSP length (of data)
                let num_bytes =
                    ptr::read_unaligned(data.add(mem::size_of::<ffi::c_uint>()) as *const [u8; 2]);
                packet.length = u16::from_ne_bytes(num_bytes);

                // check if incoming frame data length is larger than buffer length
                if packet.length as usize > mem::size_of_val(&packet.data) {
                    curr_iface.rx_error += 1;
                    libcsp_ffi::csp_can_pbuf_free(
                        if_data,
                        packet_ptr as *mut csp_packet_t,
                        1,
                        task_woken,
                    );
                    break;
                }
            }

            // reset counter
            packet.layer.rx_tx_data.rx_count = 0;

            // adjust offset to prevent CSP header from being copied to CSP data
            offset = (mem::size_of::<ffi::c_uint>() + mem::size_of::<ffi::c_ushort>()) as _;

            packet.layer.rx_tx_data.remain = (cfp_remain!(id) + 1) as u16;

            fallthrough = true;
        }
        if fallthrough || curr_frame_type == CfpFrameType::CfpMore {
            // union field accesses (no guarantee that they are initialized) and ffi calls are unsafe
            unsafe {
                // check 'remain' field match
                if (cfp_remain!(id)) != (packet.layer.rx_tx_data.remain - 1) as u32 {
                    libcsp_ffi::csp_can_pbuf_free(
                        if_data,
                        packet_ptr as *mut csp_packet_t,
                        1,
                        task_woken,
                    );
                    curr_iface.frame += 1;
                    break;
                }

                // decrement remaining frames
                packet.layer.rx_tx_data.remain -= 1;

                // improved overflow check
                let copy_length = (data_length - offset) as u16;
                if (packet.length as usize > mem::size_of_val(&packet.data))
                    || (packet.layer.rx_tx_data.rx_count + copy_length) > packet.length
                {
                    curr_iface.frame += 1;
                    libcsp_ffi::csp_can_pbuf_free(
                        if_data,
                        packet_ptr as *mut csp_packet_t,
                        1,
                        task_woken,
                    );
                    break;
                }

                // copy data_length (dlc) bytes into buffer
                let src_ptr = data.add(offset.into());
                for i in 0..copy_length {
                    packet.data.data[(packet.layer.rx_tx_data.rx_count + i) as usize] =
                        ptr::read_unaligned(src_ptr.add(i.into()));
                }
                packet.layer.rx_tx_data.rx_count += copy_length;

                // check if more data is expected
                if packet.layer.rx_tx_data.rx_count < packet.length {
                    break;
                }

                // rewrite incoming L2 broadcast to local node
                if packet.id.dst == 0x1F {
                    packet.id.dst = curr_iface.addr;
                }

                // erase from list prev->next without actually freeing buffer
                libcsp_ffi::csp_can_pbuf_free(
                    if_data,
                    packet_ptr as *mut csp_packet_t,
                    0,
                    task_woken,
                );

                // clear timestamp_rx for L3 as L2 last_used is not needed anymore
                packet.layer.rdp_data.timestamp_rx = 0;

                // signal that data is available
                libcsp_ffi::csp_qfifo_write(
                    packet_ptr as *mut csp_packet_t,
                    iface,
                    task_woken as *mut ffi::c_void,
                );
            }
        }
        break;
    }

    CspError::None
}
