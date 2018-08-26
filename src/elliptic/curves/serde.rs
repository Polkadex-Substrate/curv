pub mod serde_secret_key {
    use arithmetic::traits::Converter;
    use elliptic::curves::traits::*;
    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;
    use BigInt;
    use SK;

    #[allow(dead_code)]
    // This is not dead code, it used as part of the annotation #[serde(with = "serde_secret_key")]
    pub fn serialize<S: Serializer>(sk: &SK, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&sk.to_big_int().to_hex())
    }

    #[allow(dead_code)]
    // This is not dead code, it used as part of the annotation #[serde(with = "serde_secret_key")]
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SK, D::Error> {
        struct SecretKeyVisitor;

        impl<'de> Visitor<'de> for SecretKeyVisitor {
            type Value = SK;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("SecretKey")
            }

            fn visit_str<E: Error>(self, s: &str) -> Result<SK, E> {
                let v: SK = SK::from_big_int(&BigInt::from_hex(&String::from(s)));
                Ok(v)
            }
        }

        deserializer.deserialize_str(SecretKeyVisitor)
    }
}

pub mod serde_public_key {
    use arithmetic::traits::Converter;
    use elliptic::curves::traits::*;
    use serde::de::{MapAccess, Visitor};
    use serde::ser::SerializeStruct;
    use serde::{Deserializer, Serializer};
    use std::fmt;
    use BigInt;
    use Point;
    use PK;

    #[allow(dead_code)]
    // This is not dead code, it used as part of the annotation #[serde(with = "serde_public_key")]
    pub fn serialize<S: Serializer>(pk: &PK, serializer: S) -> Result<S::Ok, S::Error> {
        let point = pk.to_point();

        let mut state = serializer.serialize_struct("Point", 2)?;
        state.serialize_field("x", &point.x.to_hex())?;
        state.serialize_field("y", &point.y.to_hex())?;
        state.end()
    }

    #[allow(dead_code)]
    // This is not dead code, it used as part of the annotation #[serde(with = "serde_public_key")]
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PK, D::Error> {
        struct PublicKeyVisitor;

        impl<'de> Visitor<'de> for PublicKeyVisitor {
            type Value = PK;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("PublicKey")
            }

            fn visit_map<E: MapAccess<'de>>(self, mut map: E) -> Result<PK, E::Error> {
                let mut x = BigInt::from(0);
                let mut y = BigInt::from(0);

                while let Some(key) = map.next_key::<&'de str>()? {
                    let v = map.next_value::<&'de str>()?;
                    match key.as_ref() {
                        "x" => x = BigInt::from_hex(&String::from(v)),
                        "y" => y = BigInt::from_hex(&String::from(v)),
                        _ => panic!("Serialization failed!"),
                    }
                }

                Ok(PK::to_key(&Point { x, y }))
            }
        }

        deserializer.deserialize_map(PublicKeyVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::serde_public_key;
    use super::serde_secret_key;
    use elliptic::curves::traits::*;
    use serde_json;
    use BigInt;
    use EC;
    use PK;
    use SK;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct DummyStructSK {
        #[serde(with = "serde_secret_key")]
        sk: SK,
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct DummyStructPK {
        #[serde(with = "serde_public_key")]
        pk: PK,
    }

    #[test]
    fn serialize_sk() {
        let sk = SK::from_big_int(&BigInt::from(123456));
        let dummy = DummyStructSK { sk };
        let s = serde_json::to_string(&dummy).expect("Failed in serialization");
        assert_eq!(s, "{\"sk\":\"1e240\"}");
    }

    #[test]
    fn deserialize_sk() {
        let s = "{\"sk\":\"1e240\"}";
        let dummy: DummyStructSK = serde_json::from_str(s).expect("Failed in serialization");

        let sk = SK::from_big_int(&BigInt::from(123456));
        let expected_dummy = DummyStructSK { sk };

        assert_eq!(dummy, expected_dummy);
    }

    #[test]
    fn serialize_pk() {
        let slice = &[
            4, // header
            // X
            54, 57, 149, 239, 162, 148, 175, 246, 254, 239, 75, 154, 152, 10, 82, 234, 224, 85, 220,
            40, 100, 57, 121, 30, 162, 94, 156, 135, 67, 74, 49, 179, // Y
            57, 236, 53, 162, 124, 149, 144, 168, 77, 74, 30, 72, 211, 229, 110, 111, 55, 96, 193,
            86, 227, 183, 152, 195, 155, 51, 247, 123, 113, 60, 228, 188,
        ];

        let uncompressed_key = PK::from_slice(&EC::without_caps(), slice).unwrap();
        let p = uncompressed_key.to_point();

        let pk = PK::to_key(&p);
        let dummy = DummyStructPK { pk };
        let s = serde_json::to_string(&dummy).expect("Failed in serialization");
        assert_eq!(
            s,
            "{\"pk\":{\
             \"x\":\"363995efa294aff6feef4b9a980a52eae055dc286439791ea25e9c87434a31b3\",\
             \"y\":\"39ec35a27c9590a84d4a1e48d3e56e6f3760c156e3b798c39b33f77b713ce4bc\"}}"
        );
    }

    #[test]
    fn deserialize_pk() {
        let s = "{\"pk\":{\
                 \"x\":\"363995efa294aff6feef4b9a980a52eae055dc286439791ea25e9c87434a31b3\",\
                 \"y\":\"39ec35a27c9590a84d4a1e48d3e56e6f3760c156e3b798c39b33f77b713ce4bc\"}}";

        let dummy: DummyStructPK = serde_json::from_str(s).expect("Failed in serialization");

        let slice = &[
            4, // header
            // X
            54, 57, 149, 239, 162, 148, 175, 246, 254, 239, 75, 154, 152, 10, 82, 234, 224, 85, 220,
            40, 100, 57, 121, 30, 162, 94, 156, 135, 67, 74, 49, 179, // Y
            57, 236, 53, 162, 124, 149, 144, 168, 77, 74, 30, 72, 211, 229, 110, 111, 55, 96, 193,
            86, 227, 183, 152, 195, 155, 51, 247, 123, 113, 60, 228, 188,
        ];
        let uncompressed_key = PK::from_slice(&EC::without_caps(), slice).unwrap();
        let p = uncompressed_key.to_point();

        let pk_expected = PK::to_key(&p);
        assert_eq!(dummy.pk, pk_expected);
    }
}
