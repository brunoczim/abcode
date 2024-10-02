use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;

#[tokio::test]
async fn serialize_bool() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, true).await?;
    assert_eq!(buf, &[1]);

    buf.clear();
    crate::serialize(&mut buf, false).await?;
    assert_eq!(buf, &[0]);

    Ok(())
}

#[tokio::test]
async fn serialize_u8() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, 112_u8).await?;
    assert_eq!(buf, &[112]);
    Ok(())
}

#[tokio::test]
async fn serialize_i8() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, -3_i8).await?;
    assert_eq!(buf, &[0xfd]);
    Ok(())
}

#[tokio::test]
async fn serialize_u16() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, 0xe8_72_u16).await?;
    assert_eq!(buf, &[0x72, 0xe8]);
    Ok(())
}

#[tokio::test]
async fn serialize_i16() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, -0x3_i16).await?;
    assert_eq!(buf, &[0xfd, 0xff]);
    Ok(())
}

#[tokio::test]
async fn serialize_u32() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, 0x02_4c_e8_72_u32).await?;
    assert_eq!(buf, &[0x72, 0xe8, 0x4c, 0x02]);
    Ok(())
}

#[tokio::test]
async fn serialize_i32() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, -0x3_i32).await?;
    assert_eq!(buf, &[0xfd, 0xff, 0xff, 0xff,]);
    Ok(())
}

#[tokio::test]
async fn serialize_u64() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, 0x02_4c_e8_72__12_34_56_78_u64).await?;
    assert_eq!(buf, &[0x78, 0x56, 0x34, 0x12, 0x72, 0xe8, 0x4c, 0x02]);
    Ok(())
}

#[tokio::test]
async fn serialize_i64() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, -0x3_i64).await?;
    assert_eq!(buf, &[0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    Ok(())
}

#[tokio::test]
async fn serialize_u128() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(
        &mut buf,
        0x02_4c_e8_72__12_34_56_78__23_43_53_73__17_37_57_77_u128,
    )
    .await?;
    assert_eq!(
        buf,
        &[
            0x77, 0x57, 0x37, 0x17, 0x73, 0x53, 0x43, 0x23, 0x78, 0x56, 0x34,
            0x12, 0x72, 0xe8, 0x4c, 0x02,
        ]
    );
    Ok(())
}

#[tokio::test]
async fn serialize_i128() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, -0x3_i128).await?;
    assert_eq!(
        buf,
        &[
            0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff,
        ]
    );
    Ok(())
}

#[tokio::test]
async fn serialize_f32() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, 123.5_f32).await?;
    assert_eq!(buf, &(123.5_f32).to_bits().to_le_bytes());
    Ok(())
}

#[tokio::test]
async fn serialize_f64() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, 123.5_f64).await?;
    assert_eq!(buf, &(123.5_f64).to_bits().to_le_bytes());
    Ok(())
}

#[tokio::test]
async fn serialize_char() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, 'ç').await?;
    assert_eq!(buf, &[231, 0, 0, 0]);
    Ok(())
}

#[tokio::test]
async fn serialize_str() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, "façade").await?;
    assert_eq!(&buf[.. 8], &[7, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[8 ..], "façade".as_bytes());
    Ok(())
}

#[tokio::test]
async fn serialize_bytes() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, &[1_u8, 3, 2, 5] as &[u8]).await?;
    assert_eq!(&buf[.. 8], &[4, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[8 ..], &[1, 3, 2, 5]);
    Ok(())
}

#[tokio::test]
async fn serialize_none() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, Option::<u16>::None).await?;
    assert_eq!(buf, &[0]);
    Ok(())
}

#[tokio::test]
async fn serialize_some() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, Some(0x12_34_u16)).await?;
    assert_eq!(buf, &[1, 0x34, 0x12]);
    Ok(())
}

#[tokio::test]
async fn serialize_unit() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, ()).await?;
    assert_eq!(buf, &[]);
    Ok(())
}

#[tokio::test]
async fn serialize_unit_struct() -> Result<()> {
    #[derive(Debug, Clone, Copy, Serialize)]
    struct Top;

    let mut buf = Vec::new();
    crate::serialize(&mut buf, Top).await?;
    assert_eq!(buf, &[]);
    Ok(())
}

#[tokio::test]
async fn serialize_unit_variant() -> Result<()> {
    #[derive(Debug, Clone, Copy, Serialize)]
    enum Foo {
        Bar,
        Baz,
        Xyz,
    }

    let mut buf = Vec::new();
    crate::serialize(&mut buf, Foo::Bar).await?;
    assert_eq!(buf, &[0, 0, 0, 0]);

    buf.clear();
    crate::serialize(&mut buf, Foo::Baz).await?;
    assert_eq!(buf, &[1, 0, 0, 0]);

    buf.clear();
    crate::serialize(&mut buf, Foo::Xyz).await?;
    assert_eq!(buf, &[2, 0, 0, 0]);

    Ok(())
}

#[tokio::test]
async fn serialize_newtype_struct() -> Result<()> {
    #[derive(Debug, Clone, Copy, Serialize)]
    struct NewType(i32);

    let mut buf = Vec::new();
    crate::serialize(&mut buf, NewType(-4_i32)).await?;
    assert_eq!(buf, &[0xfc, 0xff, 0xff, 0xff]);
    Ok(())
}

#[tokio::test]
async fn serialize_newtype_variant() -> Result<()> {
    #[derive(Debug, Clone, Copy, Serialize)]
    enum Foo {
        Bar(bool),
        Baz(i32),
        Xyz(u8),
    }

    let mut buf = Vec::new();
    crate::serialize(&mut buf, Foo::Bar(true)).await?;
    assert_eq!(buf, &[0, 0, 0, 0, 1]);

    buf.clear();
    crate::serialize(&mut buf, Foo::Baz(-4_i32)).await?;
    assert_eq!(buf, &[1, 0, 0, 0, 0xfc, 0xff, 0xff, 0xff]);

    buf.clear();
    crate::serialize(&mut buf, Foo::Xyz(253_u8)).await?;
    assert_eq!(buf, &[2, 0, 0, 0, 253]);

    Ok(())
}

#[tokio::test]
async fn serialize_seq_empty() -> Result<()> {
    let mut buf = Vec::new();
    let sequence: Vec<&'static str> = Vec::new();
    crate::serialize(&mut buf, sequence).await?;
    assert_eq!(&buf, &[0, 0, 0, 0, 0, 0, 0, 0]);
    Ok(())
}

#[tokio::test]
async fn serialize_seq_non_empty() -> Result<()> {
    let mut buf = Vec::new();
    let sequence: Vec<&'static str> = vec!["foo!", "façade", "x"];
    crate::serialize(&mut buf, sequence).await?;
    assert_eq!(&buf[.. 8], &[3, 0, 0, 0, 0, 0, 0, 0],);
    assert_eq!(&buf[8 .. 16], &[4, 0, 0, 0, 0, 0, 0, 0],);
    assert_eq!(&buf[16 .. 20], "foo!".as_bytes());
    assert_eq!(&buf[20 .. 28], &[7, 0, 0, 0, 0, 0, 0, 0],);
    assert_eq!(&buf[28 .. 35], "façade".as_bytes());
    assert_eq!(&buf[35 .. 43], &[1, 0, 0, 0, 0, 0, 0, 0],);
    assert_eq!(&buf[43 ..], "x".as_bytes());
    Ok(())
}

#[tokio::test]
async fn serialize_tuple() -> Result<()> {
    let mut buf = Vec::new();
    crate::serialize(&mut buf, ("foo", 0x02_4c_e8_72__12_34_56_78_u64, -2_i8))
        .await?;
    assert_eq!(&buf[.. 8], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[8 .. 11], "foo".as_bytes());
    assert_eq!(
        &buf[11 .. 19],
        &[0x78, 0x56, 0x34, 0x12, 0x72, 0xe8, 0x4c, 0x02]
    );
    assert_eq!(&buf[19 ..], &[0xfe]);
    Ok(())
}

#[tokio::test]
async fn serialize_tuple_struct() -> Result<()> {
    #[derive(Debug, Clone, Copy, Serialize)]
    struct MyTuple(&'static str, u64, i8);

    let mut buf = Vec::new();
    crate::serialize(
        &mut buf,
        MyTuple("foo", 0x02_4c_e8_72__12_34_56_78_u64, -2_i8),
    )
    .await?;
    assert_eq!(&buf[.. 8], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[8 .. 11], "foo".as_bytes());
    assert_eq!(
        &buf[11 .. 19],
        &[0x78, 0x56, 0x34, 0x12, 0x72, 0xe8, 0x4c, 0x02]
    );
    assert_eq!(&buf[19 ..], &[0xfe]);

    Ok(())
}

#[tokio::test]
async fn serialize_tuple_variant() -> Result<()> {
    #[derive(Debug, Clone, Serialize)]
    enum Expr {
        Var(String),
        Apply(Box<Expr>, Box<Expr>),
        Lambda(String, Box<Expr>),
    }

    let mut buf = Vec::new();
    crate::serialize(&mut buf, Expr::Var("my_x".to_owned())).await?;
    assert_eq!(&buf[.. 4], &[0, 0, 0, 0]);
    assert_eq!(&buf[4 .. 12], &[4, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[12 ..], "my_x".as_bytes());

    buf.clear();
    crate::serialize(
        &mut buf,
        Expr::Apply(
            Box::new(Expr::Var("f".to_owned())),
            Box::new(Expr::Var("my_x".to_owned())),
        ),
    )
    .await?;
    assert_eq!(&buf[.. 4], &[1, 0, 0, 0]);
    assert_eq!(&buf[4 .. 8], &[0, 0, 0, 0]);
    assert_eq!(&buf[8 .. 16], &[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[16 .. 17], "f".as_bytes());
    assert_eq!(&buf[17 .. 21], &[0, 0, 0, 0]);
    assert_eq!(&buf[21 .. 29], &[4, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[29 ..], "my_x".as_bytes());

    buf.clear();
    crate::serialize(
        &mut buf,
        Expr::Lambda("y_".to_owned(), Box::new(Expr::Var("x".to_owned()))),
    )
    .await?;
    assert_eq!(&buf[.. 4], &[2, 0, 0, 0]);
    assert_eq!(&buf[4 .. 12], &[2, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[12 .. 14], "y_".as_bytes());
    assert_eq!(&buf[14 .. 18], &[0, 0, 0, 0]);
    assert_eq!(&buf[18 .. 26], &[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[26 ..], "x".as_bytes());

    Ok(())
}

#[tokio::test]
async fn serialize_map_empty() -> Result<()> {
    let mut buf = Vec::new();
    let sequence = BTreeMap::<&'static str, u16>::new();
    crate::serialize(&mut buf, sequence).await?;
    assert_eq!(buf, &[0, 0, 0, 0, 0, 0, 0, 0]);
    Ok(())
}

#[tokio::test]
async fn serialize_map_non_empty() -> Result<()> {
    let mut buf = Vec::new();
    let mut sequence = BTreeMap::<&'static str, u16>::new();
    sequence.insert("aoo", 12);
    sequence.insert("b", 0x4a_bc);
    sequence.insert("cigger", 134);
    sequence.insert("dreater", 1);
    crate::serialize(&mut buf, sequence).await?;
    assert_eq!(&buf[.. 8], &[4, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[8 .. 16], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[16 .. 19], "aoo".as_bytes());
    assert_eq!(&buf[19 .. 21], &[12, 0]);
    assert_eq!(&buf[21 .. 29], &[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[29 .. 30], "b".as_bytes());
    assert_eq!(&buf[30 .. 32], &[0xbc, 0x4a]);
    assert_eq!(&buf[32 .. 40], &[6, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[40 .. 46], "cigger".as_bytes());
    assert_eq!(&buf[46 .. 48], &[134, 0]);
    assert_eq!(&buf[48 .. 56], &[7, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[56 .. 63], "dreater".as_bytes());
    assert_eq!(&buf[63 ..], &[1, 0]);
    Ok(())
}

#[tokio::test]
async fn serialize_struct() -> Result<()> {
    #[derive(Debug, Clone, Copy, Serialize)]
    struct MyStruct {
        name: &'static str,
        foo: u64,
        bar: i8,
    }

    let mut buf = Vec::new();
    crate::serialize(
        &mut buf,
        MyStruct {
            name: "foo",
            foo: 0x02_4c_e8_72__12_34_56_78_u64,
            bar: -2_i8,
        },
    )
    .await?;
    assert_eq!(&buf[.. 8], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[8 .. 11], "foo".as_bytes());
    assert_eq!(
        &buf[11 .. 19],
        &[0x78, 0x56, 0x34, 0x12, 0x72, 0xe8, 0x4c, 0x02]
    );
    assert_eq!(&buf[19 ..], &[0xfe]);

    Ok(())
}

#[tokio::test]
async fn serialize_struct_variant() -> Result<()> {
    #[derive(Debug, Clone, Serialize)]
    enum Expr {
        Var { ident: String },
        Apply { fun: Box<Expr>, arg: Box<Expr> },
        Lambda { param: String, body: Box<Expr> },
    }

    let mut buf = Vec::new();
    crate::serialize(&mut buf, Expr::Var { ident: "my_x".to_owned() }).await?;
    assert_eq!(&buf[.. 4], &[0, 0, 0, 0]);
    assert_eq!(&buf[4 .. 12], &[4, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[12 ..], "my_x".as_bytes());

    buf.clear();
    crate::serialize(
        &mut buf,
        Expr::Apply {
            fun: Box::new(Expr::Var { ident: "f".to_owned() }),
            arg: Box::new(Expr::Var { ident: "my_x".to_owned() }),
        },
    )
    .await?;
    assert_eq!(&buf[.. 4], &[1, 0, 0, 0]);
    assert_eq!(&buf[4 .. 8], &[0, 0, 0, 0]);
    assert_eq!(&buf[8 .. 16], &[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[16 .. 17], "f".as_bytes());
    assert_eq!(&buf[17 .. 21], &[0, 0, 0, 0]);
    assert_eq!(&buf[21 .. 29], &[4, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[29 ..], "my_x".as_bytes());

    buf.clear();
    crate::serialize(
        &mut buf,
        Expr::Lambda {
            param: "y_".to_owned(),
            body: Box::new(Expr::Var { ident: "x".to_owned() }),
        },
    )
    .await?;
    assert_eq!(&buf[.. 4], &[2, 0, 0, 0]);
    assert_eq!(&buf[4 .. 12], &[2, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[12 .. 14], "y_".as_bytes());
    assert_eq!(&buf[14 .. 18], &[0, 0, 0, 0]);
    assert_eq!(&buf[18 .. 26], &[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[26 ..], "x".as_bytes());

    Ok(())
}

#[tokio::test]
async fn serialize_into_buffer() -> Result<()> {
    #[derive(Debug, Clone, Serialize)]
    struct MyStruct {
        name: &'static str,
        foo: u64,
        ids: Vec<Vec<i32>>,
        bar: i8,
    }

    let value = MyStruct {
        name: "foo",
        foo: 0x02_4c_e8_72__12_34_56_78_u64,
        ids: vec![vec![1, 2, 3], vec![-2, 0x3_f1_f2], vec![]],
        bar: -2_i8,
    };
    let buf = crate::serialize_into_buffer(value)?;

    assert_eq!(&buf[.. 8], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[8 .. 11], "foo".as_bytes());
    assert_eq!(
        &buf[11 .. 19],
        &[0x78, 0x56, 0x34, 0x12, 0x72, 0xe8, 0x4c, 0x02]
    );
    assert_eq!(&buf[19 .. 27], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[27 .. 35], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[35 .. 39], &[1, 0, 0, 0]);
    assert_eq!(&buf[39 .. 43], &[2, 0, 0, 0]);
    assert_eq!(&buf[43 .. 47], &[3, 0, 0, 0]);
    assert_eq!(&buf[47 .. 55], &[2, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[55 .. 59], &[0xfe, 0xff, 0xff, 0xff]);
    assert_eq!(&buf[59 .. 63], &[0xf2, 0xf1, 0x3, 0]);
    assert_eq!(&buf[63 .. 71], &[0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[71 ..], &[0xfe]);

    Ok(())
}

#[tokio::test]
async fn serialize_on_buffer() -> Result<()> {
    #[derive(Debug, Clone, Serialize)]
    struct MyStruct {
        name: &'static str,
        foo: u64,
        ids: Vec<Vec<i32>>,
        bar: i8,
    }

    let value = MyStruct {
        name: "foo",
        foo: 0x02_4c_e8_72__12_34_56_78_u64,
        ids: vec![vec![1, 2, 3], vec![-2, 0x3_f1_f2], vec![]],
        bar: -2_i8,
    };
    let mut buf = Vec::new();
    crate::serialize_on_buffer(&mut buf, value)?;

    assert_eq!(&buf[.. 8], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[8 .. 11], "foo".as_bytes());
    assert_eq!(
        &buf[11 .. 19],
        &[0x78, 0x56, 0x34, 0x12, 0x72, 0xe8, 0x4c, 0x02]
    );
    assert_eq!(&buf[19 .. 27], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[27 .. 35], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[35 .. 39], &[1, 0, 0, 0]);
    assert_eq!(&buf[39 .. 43], &[2, 0, 0, 0]);
    assert_eq!(&buf[43 .. 47], &[3, 0, 0, 0]);
    assert_eq!(&buf[47 .. 55], &[2, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[55 .. 59], &[0xfe, 0xff, 0xff, 0xff]);
    assert_eq!(&buf[59 .. 63], &[0xf2, 0xf1, 0x3, 0]);
    assert_eq!(&buf[63 .. 71], &[0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&buf[71 ..], &[0xfe]);

    Ok(())
}
