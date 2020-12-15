/*
    Copyright 2020 Sojan James
    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at
        http://www.apache.org/licenses/LICENSE-2.0
    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

/// User and group ID that will be used.  This should probably go into a crate
/// of its own so that it can be used by other applcations.

pub struct Ids;

impl Ids {
    pub fn get_uid(id: &str) -> Option<u32> {
        if let Some((uid, _gid)) = Self::get_uid_gid(id) {
            Some(uid)
        } else {
            None
        }
    }

    pub fn get_gid(id: &str) -> Option<u32> {
        if let Some((_uid, gid)) = Self::get_uid_gid(id) {
            Some(gid)
        } else {
            None
        }
    }

    pub fn get_uid_gid(id: &str) -> Option<(u32, u32)> {
        match id {
            "root" => Some((0, 0)),
            "system" => Some((1000, 1000)),
            "radio" => Some((1002, 1002)),
            "bluetooth" => Some((1002, 1002)),
            "graphics" => Some((1003, 1003)),
            "input" => Some((1004, 1004)),
            "audio" => Some((1005, 1005)),
            "camera" => Some((1006, 1006)),
            "log" => Some((1007, 1007)),
            "compass" => Some((1008, 1008)),
            "mount" => Some((1009, 1009)),
            "wifi" => Some((1010, 1010)),
            "debug" => Some((1011, 1011)),
            "install" => Some((1012, 1012)),
            "media" => Some((1013, 1013)),
            "dhcp" => Some((1014, 1014)),
            "vpn" => Some((1016, 1016)),
            "keystore" => Some((1017, 1017)),
            "usb" => Some((1018, 1018)),
            "drm" => Some((1019, 1019)),
            "mdns" => Some((1020, 1020)),
            "gps" => Some((1021, 1021)),
            "media_rw" => Some((1023, 1023)),
            "mtp" => Some((1024, 1024)),
            "nfc" => Some((1027, 1027)),
            "shell" => Some((2000, 2000)),
            "diag" => Some((2002, 2002)),
            "net_bt_admin" => Some((3001, 3001)),
            "net_bt" => Some((3002, 3002)),
            "net_raw" => Some((3004, 3004)),
            "net_admin" => Some((3005, 3005)),
            "net_bw_stats" => Some((3006, 3006)),
            "net_bw_acct" => Some((3007, 3007)),
            "net_bt_stack" => Some((3008, 3008)),
            "reserved1" => Some((3009, 3009)),
            "reserved2" => Some((3010, 3010)),
            "reserved3" => Some((3011, 3011)),
            "reserved4" => Some((3012, 3012)),
            "reserved5" => Some((3013, 3013)),
            "reserved6" => Some((3014, 3014)),
            "nobody" => Some((9999, 9999)),
            _ => None,
        }
    }
}
