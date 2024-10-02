use std::collections::BTreeMap;

use anyhow::Result;
use serde::Deserialize;

#[tokio::test]
async fn deserialize_bool() -> Result<()> {
    let buf = [0_u8];
    let value: bool = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, false);

    let buf = [1_u8];
    let value: bool = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, true);

    Ok(())
}

#[tokio::test]
async fn deserialize_u8() -> Result<()> {
    let buf = [123_u8];
    let value: u8 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, 123);
    Ok(())
}

#[tokio::test]
async fn deserialize_i8() -> Result<()> {
    let buf = [0xfd_u8];
    let value: i8 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, -3);
    Ok(())
}

#[tokio::test]
async fn deserialize_u16() -> Result<()> {
    let buf = [0xab_u8, 0xcd];
    let value: u16 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, 0xcd_ab);
    Ok(())
}

#[tokio::test]
async fn deserialize_i16() -> Result<()> {
    let buf = [0xfd_u8, 0xff];
    let value: i16 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, -3);
    Ok(())
}

#[tokio::test]
async fn deserialize_u32() -> Result<()> {
    let buf = [0xab_u8, 0xcd, 0x12, 0x34];
    let value: u32 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, 0x34_12_cd_ab);
    Ok(())
}

#[tokio::test]
async fn deserialize_i32() -> Result<()> {
    let buf = [0xfd_u8, 0xff, 0xff, 0xff];
    let value: i32 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, -3);

    Ok(())
}

#[tokio::test]
async fn deserialize_u64() -> Result<()> {
    let buf = [0x1a_u8, 0xef, 0x78, 0x56, 0xab, 0xcd, 0x12, 0x34];
    let value: u64 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, 0x34_12_cd_ab__56_78_ef_1a);
    Ok(())
}

#[tokio::test]
async fn deserialize_i64() -> Result<()> {
    let buf = [0xfd_u8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let value: i64 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, -3);
    Ok(())
}

#[tokio::test]
async fn deserialize_u128() -> Result<()> {
    let buf = [
        0x73_u8, 0xd2, 0xe1, 0xf0, 0xe5, 0xd4, 0xc3, 0xb2, 0x1a, 0xef, 0x78,
        0x56, 0xab, 0xcd, 0x12, 0x34,
    ];
    let value: u128 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, 0x34_12_cd_ab__56_78_ef_1a__b2_c3_d4_e5__f0_e1_d2_73);
    Ok(())
}

#[tokio::test]
async fn deserialize_i128() -> Result<()> {
    let buf = [
        0xfd_u8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    let value: i128 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, -3);
    Ok(())
}

#[tokio::test]
async fn deserialize_f32() -> Result<()> {
    let buf = &(123.5_f32).to_bits().to_le_bytes();
    let value: f32 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, 123.5);
    Ok(())
}

#[tokio::test]
async fn deserialize_f64() -> Result<()> {
    let buf = &(123.5_f64).to_bits().to_le_bytes();
    let value: f64 = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, 123.5);
    Ok(())
}

#[tokio::test]
async fn deserialize_char() -> Result<()> {
    let buf = u32::from('ç').to_le_bytes();
    let value: char = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, 'ç');
    Ok(())
}

#[tokio::test]
async fn deserialize_string() -> Result<()> {
    let mut buf = [0_u8; 15];
    buf[.. 8].copy_from_slice(&[7, 0, 0, 0, 0, 0, 0, 0]);
    buf[8 ..].copy_from_slice("façade".as_bytes());
    let value: String = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, "façade");
    Ok(())
}

#[tokio::test]
async fn deserialize_vec() -> Result<()> {
    let mut buf = [0_u8; 13];
    buf[.. 8].copy_from_slice(&[5, 0, 0, 0, 0, 0, 0, 0]);
    buf[8 ..].copy_from_slice(&[1, 3, 2, 5, 4]);
    let value: Vec<u8> = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, &[1, 3, 2, 5, 4]);
    Ok(())
}

#[tokio::test]
async fn deserialize_none() -> Result<()> {
    let buf = [0_u8];
    let value: Option<i16> = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, None);
    Ok(())
}

#[tokio::test]
async fn deserialize_some() -> Result<()> {
    let buf = [1_u8, 0xfb, 0x9];
    let value: Option<i16> = crate::deserialize(&buf[..] as &[_]).await?;
    assert_eq!(value, Some(0x9_fb));
    Ok(())
}

#[tokio::test]
async fn deserialize_unit() -> Result<()> {
    let buf: &[u8] = &[];
    let value: () = crate::deserialize(buf).await?;
    assert_eq!(value, ());
    Ok(())
}

#[tokio::test]
async fn deserialize_unit_struct() -> Result<()> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
    struct MyUnit;

    let buf: &[u8] = &[];
    let value: MyUnit = crate::deserialize(buf).await?;
    assert_eq!(value, MyUnit);
    Ok(())
}

#[tokio::test]
async fn deserialize_newtype_struct() -> Result<()> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
    struct NewWord(u16);

    let buf: &[u8] = &[0xab, 0xcd];
    let value: NewWord = crate::deserialize(buf).await?;
    assert_eq!(value, NewWord(0xcd_ab));
    Ok(())
}

#[tokio::test]
async fn deserialize_seq_empty() -> Result<()> {
    let buf = [0; 8];
    let value: Vec<i16> = crate::deserialize(&buf[..]).await?;
    assert_eq!(value, &[]);
    Ok(())
}

#[tokio::test]
async fn deserialize_seq_non_empty() -> Result<()> {
    let mut buf = [0; 14];
    buf[.. 8].copy_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0]);
    buf[8 .. 10].copy_from_slice(&[0xfd, 0xff]);
    buf[10 .. 12].copy_from_slice(&[0xfd, 0xf]);
    buf[12 ..].copy_from_slice(&[0x1, 0x0]);
    let value: Vec<i16> = crate::deserialize(&buf[..]).await?;
    assert_eq!(value, &[-3, 0xf_fd, 1]);
    Ok(())
}

#[tokio::test]
async fn deserialize_tuple() -> Result<()> {
    let mut buf = [0; 14];
    buf[.. 8].copy_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0]);
    buf[8 .. 11].copy_from_slice("foo".as_bytes());
    buf[11 .. 12].copy_from_slice(&[9]);
    buf[12 .. 14].copy_from_slice(&[0x3f, 0xa]);
    let value: (String, bool, u16) = crate::deserialize(&buf[..]).await?;
    assert_eq!(value, ("foo".to_owned(), true, 0xa_3f));
    Ok(())
}

#[tokio::test]
async fn deserialize_tuple_struct() -> Result<()> {
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    struct MyTuple(String, bool, u16);

    let mut buf = [0; 14];
    buf[.. 8].copy_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0]);
    buf[8 .. 11].copy_from_slice("foo".as_bytes());
    buf[11 .. 12].copy_from_slice(&[9]);
    buf[12 .. 14].copy_from_slice(&[0x3f, 0xa]);
    let value: MyTuple = crate::deserialize(&buf[..]).await?;
    assert_eq!(value, MyTuple("foo".to_owned(), true, 0xa_3f));
    Ok(())
}

#[tokio::test]
async fn deserialize_map_empty() -> Result<()> {
    let buf = [0; 8];
    let value: BTreeMap<String, i16> = crate::deserialize(&buf[..]).await?;
    assert_eq!(value, BTreeMap::new());
    Ok(())
}

#[tokio::test]
async fn deserialize_map_non_empty() -> Result<()> {
    let mut buf = [0; 35];
    buf[.. 8].copy_from_slice(&[2, 0, 0, 0, 0, 0, 0, 0]);
    buf[8 .. 16].copy_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0]);
    buf[16 .. 19].copy_from_slice("xyz".as_bytes());
    buf[19 .. 21].copy_from_slice(&[0xfd, 0xf]);
    buf[21 .. 29].copy_from_slice(&[4, 0, 0, 0, 0, 0, 0, 0]);
    buf[29 .. 33].copy_from_slice("abcd".as_bytes());
    buf[33 ..].copy_from_slice(&[0x1, 0x0]);
    let value: BTreeMap<String, i16> = crate::deserialize(&buf[..]).await?;
    assert_eq!(value, {
        let mut map = BTreeMap::new();
        map.insert("xyz".to_owned(), 0xf_fd);
        map.insert("abcd".to_owned(), 1);
        map
    });
    Ok(())
}

#[tokio::test]
async fn deserialize_struct() -> Result<()> {
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    struct MyStruct {
        name: String,
        active: bool,
        id: u16,
    }

    let mut buf = [0; 14];
    buf[.. 8].copy_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0]);
    buf[8 .. 11].copy_from_slice("foo".as_bytes());
    buf[11 .. 12].copy_from_slice(&[9]);
    buf[12 .. 14].copy_from_slice(&[0x3f, 0xa]);
    let value: MyStruct = crate::deserialize(&buf[..]).await?;
    assert_eq!(
        value,
        MyStruct { name: "foo".to_owned(), active: true, id: 0xa_3f }
    );
    Ok(())
}

#[tokio::test]
async fn deserialize_enum() -> Result<()> {
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    enum Stuff {
        Foo { x: String, y: u32 },
        Bar(u8, i32, bool),
        Baz,
    }

    let mut buf = [0; 18];
    buf[.. 4].copy_from_slice(&[0, 0, 0, 0]);
    buf[4 .. 12].copy_from_slice(&[2, 0, 0, 0, 0, 0, 0, 0]);
    buf[12 .. 14].copy_from_slice("xy".as_bytes());
    buf[14 .. 18].copy_from_slice(&[0x78, 0xef, 0xcd, 0xab]);
    let value: Stuff = crate::deserialize(&buf[..]).await?;
    assert_eq!(value, Stuff::Foo { x: "xy".to_owned(), y: 0xab_cd_ef_78 });

    let mut buf = [0; 18];
    buf[.. 4].copy_from_slice(&[1, 0, 0, 0]);
    buf[4 .. 5].copy_from_slice(&[123]);
    buf[5 .. 9].copy_from_slice(&[0xcd, 0xac, 0xbd, 0x1b]);
    buf[9 .. 10].copy_from_slice(&[0]);
    let value: Stuff = crate::deserialize(&buf[..]).await?;
    assert_eq!(value, Stuff::Bar(123, 0x_1b_bd_ac_cd, false));

    let buf: [u8; 4] = [2, 0, 0, 0];
    let value: Stuff = crate::deserialize(&buf[..]).await?;
    assert_eq!(value, Stuff::Baz);

    Ok(())
}

#[tokio::test]
async fn unexpected_eof() -> Result<()> {
    let buf: &[u8] = &[];
    let result: Result<u8, _> =
        crate::de::Config::default().with_hard_eof().deserialize(buf).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn expected_eof() -> Result<()> {
    let buf: &[u8] = &[1, 2];
    let result: Result<u8, _> =
        crate::de::Config::default().with_hard_eof().deserialize(buf).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn deserialize_struct_synchronous() -> Result<()> {
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    struct MyStruct {
        name: String,
        active: bool,
        id: u16,
    }

    let mut buf = [0; 14];
    buf[.. 8].copy_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0]);
    buf[8 .. 11].copy_from_slice("foo".as_bytes());
    buf[11 .. 12].copy_from_slice(&[9]);
    buf[12 .. 14].copy_from_slice(&[0x3f, 0xa]);
    let value: MyStruct = crate::deserialize_buffer(&buf[..])?;
    assert_eq!(
        value,
        MyStruct { name: "foo".to_owned(), active: true, id: 0xa_3f }
    );
    Ok(())
}
