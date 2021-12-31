mod type_id;

const MAX_TUPLE: u8 = 250;
// const MAX_TABLE_NESTING: u8 = 250;
// lua internals
// const MAX_BITS: u32 = 26;
// const MAX_ARRAY_SIZE: u32 = 1 << MAX_BITS;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Key {
    Boolean(bool),
    Number(ordered_float::NotNan<f64>),
    String(String),
    Table(Vec<(Key, Value)>),
}

impl Key {
    pub fn get_bool(&self) -> Option<bool> {
        match self {
            Key::Boolean(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn get_number(&self) -> Option<ordered_float::NotNan<f64>> {
        match self {
            Key::Number(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<&str> {
        match self {
            Key::String(inner) => Some(inner.as_str()),
            _ => None,
        }
    }

    pub fn get_table(&self) -> Option<&[(Key, Value)]> {
        match self {
            Key::Table(inner) => Some(inner.as_slice()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Table(Vec<(Key, Value)>),
}

impl Value {
    pub fn get_nil(&self) -> Option<()> {
        match self {
            Value::Nil => Some(()),
            _ => None,
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn get_number(&self) -> Option<f64> {
        match self {
            Value::Number(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<&str> {
        match self {
            Value::String(inner) => Some(inner.as_str()),
            _ => None,
        }
    }

    pub fn get_table(&self) -> Option<&[(Key, Value)]> {
        match self {
            Value::Table(inner) => Some(inner.as_slice()),
            _ => None,
        }
    }
}

fn load_element_count(data: &[u8]) -> nom::IResult<&[u8], u8> {
    let (data, count) = nom::number::complete::u8(data)?;
    if count > MAX_TUPLE {
        Err(nom::Err::Error(nom::error::make_error(
            data,
            nom::error::ErrorKind::LengthValue,
        )))
    } else {
        Ok((data, count))
    }
}

fn load_type_id(data: &[u8]) -> nom::IResult<&[u8], type_id::TypeIdentifier> {
    nom::combinator::map_res(nom::number::complete::u8, std::convert::TryInto::try_into)(data)
}

fn load_key_value(data: &[u8]) -> nom::IResult<&[u8], (Key, Value)> {
    let (data, key) = load_key(data)?;
    let (data, value) = load_value(data)?;
    Ok((data, (key, value)))
}

fn load_string(data: &[u8]) -> nom::IResult<&[u8], String> {
    let (data, count) = nom::number::complete::le_u32(data)?;
    let (data, str) =
        nom::combinator::map_res(nom::bytes::complete::take(count), std::str::from_utf8)(data)?;
    Ok((data, str.to_owned()))
}

fn load_table(data: &[u8]) -> nom::IResult<&[u8], Vec<(Key, Value)>> {
    let (data, array_size) = nom::number::complete::le_u32(data)?;
    let (data, hash_size) = nom::number::complete::le_u32(data)?;
    let total_size = array_size + hash_size;
    // TODO: validation

    nom::multi::count(load_key_value, total_size as usize)(data)
}

fn load_key(data: &[u8]) -> nom::IResult<&[u8], Key> {
    use type_id::TypeIdentifier;

    let (data, ty) = load_type_id(data)?;
    match ty {
        TypeIdentifier::NIL => Err(nom::Err::Error(nom::error::make_error(
            data,
            nom::error::ErrorKind::Digit,
        ))),
        TypeIdentifier::FALSE => Ok((data, Key::Boolean(false))),
        TypeIdentifier::TRUE => Ok((data, Key::Boolean(true))),
        TypeIdentifier::NUMBER => {
            fn parse_non_nan(n: f64) -> Result<Key, nom::error::ErrorKind> {
                let n = std::convert::TryFrom::try_from(n)
                    .map_err(|_err| nom::error::ErrorKind::Digit)?;
                Ok(Key::Number(n))
            }
            nom::combinator::map_res(nom::number::complete::le_f64, parse_non_nan)(data)
        }
        TypeIdentifier::STRING => load_string(data).map(|(data, value)| (data, Key::String(value))),
        TypeIdentifier::TABLE => load_table(data).map(|(data, table)| (data, Key::Table(table))),
    }
}

fn load_value(data: &[u8]) -> nom::IResult<&[u8], Value> {
    use type_id::TypeIdentifier;

    let (data, ty) = load_type_id(data)?;
    match ty {
        TypeIdentifier::NIL => Ok((data, Value::Nil)),
        TypeIdentifier::FALSE => Ok((data, Value::Boolean(false))),
        TypeIdentifier::TRUE => Ok((data, Value::Boolean(true))),
        TypeIdentifier::NUMBER => {
            nom::combinator::map(nom::number::complete::le_f64, |n| Value::Number(n.into()))(data)
        }
        TypeIdentifier::STRING => {
            load_string(data).map(|(data, string)| (data, Value::String(string)))
        }
        TypeIdentifier::TABLE => load_table(data).map(|(data, table)| (data, Value::Table(table))),
    }
}

fn save_table(result: &mut Vec<u8>, table: &[(Key, Value)]) {
    // The canonical implementation of this function is here
    // https://github.com/lua/lua/blob/ad3942adba574c9d008c99ce2785a5af19d146bf/ltable.c#L889-L966
    fn array_size(table: &[(Key, Value)]) -> usize {
        let mut size = 0;
        for index in 1..=(table.len()) {
            let v = table.iter().find(|(key, _value)| {
                if let Some(v) = key.get_number() {
                    index == v.into_inner() as usize
                } else {
                    false
                }
            });
            if v.is_some() {
                size = index;
            } else {
                break;
            }
        }
        size
    }

    let array = array_size(table);
    let hash_size = table.len() - array;
    result.push(type_id::TypeIdentifier::TABLE as u8);
    result.extend_from_slice(&((array as u32).to_le_bytes()));
    result.extend_from_slice(&((hash_size as u32).to_le_bytes()));

    // TODO: validate nesting depth
    for (key, value) in table {
        save_key(result, key);
        save_value(result, value);
    }
}

fn save_key(result: &mut Vec<u8>, key: &Key) {
    match key {
        Key::Boolean(inner) => match *inner {
            true => result.push(type_id::TypeIdentifier::TRUE as u8),
            false => result.push(type_id::TypeIdentifier::FALSE as u8),
        },
        Key::Number(inner) => {
            result.push(type_id::TypeIdentifier::NUMBER as u8);
            result.extend_from_slice(&inner.into_inner().to_le_bytes());
        }
        Key::String(inner) => {
            result.push(type_id::TypeIdentifier::STRING as u8);
            result.extend_from_slice(&(inner.len() as u32).to_le_bytes());
            result.extend_from_slice(inner.as_bytes());
        }
        Key::Table(table) => save_table(result, table),
    }
}

fn save_value(result: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Nil => result.push(type_id::TypeIdentifier::NIL as u8),
        Value::Boolean(inner) => match *inner {
            true => result.push(type_id::TypeIdentifier::TRUE as u8),
            false => result.push(type_id::TypeIdentifier::FALSE as u8),
        },
        Value::Number(inner) => {
            result.push(type_id::TypeIdentifier::NUMBER as u8);
            result.extend_from_slice(&inner.to_le_bytes());
        }
        Value::String(inner) => {
            result.push(type_id::TypeIdentifier::STRING as u8);
            result.extend_from_slice(&(inner.len() as u32).to_le_bytes());
            result.extend_from_slice(inner.as_bytes());
        }
        Value::Table(table) => save_table(result, table),
    }
}

pub fn load(data: &[u8]) -> nom::IResult<&[u8], Vec<Value>> {
    nom::multi::length_count(load_element_count, load_value)(data)
}

pub fn save(data: &[Value]) -> Vec<u8> {
    let mut result = Vec::new();
    result.push(data.len() as u8);
    for datum in data {
        save_value(&mut result, datum);
    }
    result
}
