// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use super::*;

/// template definition decoder
pub trait DefinitionDecoder {
    type Item;
    type Error;

    /// set member count;
    /// to decode the definition, it need to specify the number of members.
    fn member_count(&mut self, member_count: u16);

    /// partial decode
    fn partial_decode(&mut self, buf: Bytes) -> StdResult<(), Self::Error>;

    /// finally decode
    fn decode(&mut self) -> StdResult<Self::Item, Self::Error>;
}

/// default template definition decoder
#[derive(Debug, Default)]
pub struct DefaultDefinitionDecoder {
    /// template name
    name: String,
    /// members of template
    members: SmallVec<[MemberInfo; 8]>,
    /// the exact number of members
    member_count: u16,
    /// index to track when decode member names
    index: u16,
}

impl DefinitionDecoder for DefaultDefinitionDecoder {
    type Error = Error;
    type Item = TemplateDefinition;

    fn member_count(&mut self, member_count: u16) {
        self.member_count = member_count;
    }

    fn partial_decode(&mut self, mut buf: Bytes) -> StdResult<(), Self::Error> {
        if self.member_count < 2 {
            return Err(invalid_data(
                "template definition - need to initialize `member_count`",
            ));
        }
        while self.members.len() < self.member_count as usize {
            //TODO: validate buf.len()
            let item = MemberInfo {
                name: Default::default(),
                array_size: buf.get_u16_le(),
                type_info: SymbolType(buf.get_u16_le()),
                offset: buf.get_u32_le(),
            };
            self.members.push(item);
        }
        let mut strings = buf.split(|v| *v == 0);
        if self.name.is_empty() {
            if let Some(buf) = strings.next() {
                self.name = get_name(buf);
            }
        }
        for buf in strings {
            if self.index < self.member_count {
                self.members[self.index as usize].name = get_name(buf);
                self.index += 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    /// finally decode, return the target object and reset inner state of the decoder
    fn decode(&mut self) -> StdResult<Self::Item, Self::Error> {
        if self.member_count < 2 {
            return Err(invalid_data(
                "template definition - need to initialize `member_count`",
            ));
        }
        if self.index < self.member_count {
            return Err(invalid_data(
                "template definition - not enough data to decode",
            ));
        }
        self.index = 0;
        self.member_count = 0;
        let map: HashMap<_, _> = self
            .members
            .drain(..)
            .map(|item| (item.name.clone(), item))
            .collect();
        Ok(TemplateDefinition {
            name: mem::take(&mut self.name),
            members: map,
        })
    }
}

/// name might contains `;`,  truncate to get the name
fn get_name(buf: &[u8]) -> String {
    // split by semi-colon
    let mut parts = buf.split(|v| *v == 0x3B);
    let name_buf = parts.next().unwrap();
    String::from_utf8_lossy(name_buf).into_owned().into()
}
