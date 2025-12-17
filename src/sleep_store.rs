#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LastLonLat {
    pub last_lat: Option<f64>,
    pub last_lon: Option<f64>,
}

#[repr(C)]
struct RtcMemory {
    magic: u32,
    data: LastLonLat,
}

const MAGIC_NUMBER: u32 = 0x4750_5321; // "GPS!" in hex-ish

#[link_section = ".rtc.noinit"]
static mut RTC_MEM: RtcMemory = RtcMemory {
    magic: 0,
    data: LastLonLat {
        last_lat: None,
        last_lon: None,
    },
};

/// Safe wrapper for RTC memory operations
pub struct DeepSleepStore;

impl DeepSleepStore {
    /// Safely load data from RTC memory.
    /// Returns default/empty data if the magic number doesn't match (first boot).
    pub fn load() -> LastLonLat {
        unsafe {
            if RTC_MEM.magic == MAGIC_NUMBER {
                RTC_MEM.data
            } else {
                LastLonLat {
                    last_lat: None,
                    last_lon: None,
                }
            }
        }
    }

    /// Safely save data to RTC memory.
    pub fn save(data: LastLonLat) {
        unsafe {
            RTC_MEM.data = data;
            RTC_MEM.magic = MAGIC_NUMBER;
        }
    }
}
