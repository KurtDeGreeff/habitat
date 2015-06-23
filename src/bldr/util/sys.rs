//
// Copyright:: Copyright (c) 2015 Chef Software, Inc.
// License:: Apache License, Version 2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use error::{BldrResult, BldrError};
use std::process::Command;
use std::collections::BTreeMap;
use toml;

pub fn ip() -> BldrResult<String> {
    debug!("Shelling out to determine IP address");
    let output = try!(Command::new("sh")
        .arg("-c")
        .arg("ip route get 8.8.8.8 | awk '{printf \"%s\", $NF; exit}'")
        .output());
    match output.status.success() {
        true => {
            debug!("IP address is {}", String::from_utf8_lossy(&output.stdout));
            let ip = try!(String::from_utf8(output.stdout));
            Ok(ip)
        },
        false => {
            debug!("IP address command returned: OUT: {} ERR: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            Err(BldrError::IPFailed)
        },
    }
}

pub fn hostname() -> BldrResult<String> {
    debug!("Shelling out to determine IP address");
    let output = try!(Command::new("sh")
        .arg("-c")
        .arg("hostname | awk '{printf \"%s\", $NF; exit}'")
        .output());
    match output.status.success() {
        true => {
            debug!("Hostname address is {}", String::from_utf8_lossy(&output.stdout));
            let hostname = try!(String::from_utf8(output.stdout));
            Ok(hostname)
        },
        false => {
            debug!("Hostname address command returned: OUT: {} ERR: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            Err(BldrError::IPFailed)
        },
    }
}

pub fn to_toml() -> BldrResult<BTreeMap<String, toml::Value>> {
    let mut toml_string = String::new();
    let ip = try!(ip());
    toml_string.push_str(&format!("ip = \"{}\"\n", ip));
    let hostname = try!(hostname());
    toml_string.push_str(&format!("hostname = \"{}\"\n", hostname));
    debug!("Sys Toml: {}", toml_string);
    let mut toml_parser = toml::Parser::new(&toml_string);
    let toml_value = try!(toml_parser.parse().ok_or(BldrError::TomlParser(toml_parser.errors)));
    Ok(toml_value)
}