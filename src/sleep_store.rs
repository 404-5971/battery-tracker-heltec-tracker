// use log::info;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LastLonLat {
    pub lat: f32,
    pub lon: f32,
    pub valid: u8, // 0 = false, 1 = true
}

#[repr(C)]
struct RtcMemory {
    magic: u32,
    data: LastLonLat,
}

const MAGIC_NUMBER: u32 = 0x4750_5321; // "GPS!" in hex-ish

#[no_mangle]
#[link_section = ".rtc_noinit"]
static mut RTC_MEM: RtcMemory = RtcMemory {
    magic: 0,
    data: LastLonLat {
        lat: 0.0,
        lon: 0.0,
        valid: 0,
    },
};

/// Safe wrapper for RTC memory operations
pub struct DeepSleepStore;

impl DeepSleepStore {
    /// Safely load data from RTC memory.
    pub fn load() -> Option<(f32, f32)> {
        unsafe {
            let magic = RTC_MEM.magic;
            let data = RTC_MEM.data;
            // let addr = std::ptr::addr_of!(RTC_MEM);

            // info!("RTC Memory Address: {:p}", addr);
            // info!("RTC Magic: {:#x}", magic);
            // info!("RTC Data: {:?}", data);

            if magic == MAGIC_NUMBER && data.valid == 1 {
                Some((data.lat, data.lon))
            } else {
                None
            }
        }
    }

    /// Safely save data to RTC memory.
    pub fn save(lat: f32, lon: f32) {
        unsafe {
            // info!("Saving to RTC: Lat: {}, Lon: {}", lat, lon);
            RTC_MEM.data = LastLonLat { lat, lon, valid: 1 };
            RTC_MEM.magic = MAGIC_NUMBER;
        }
    }
}
