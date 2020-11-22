use bincode::{deserialize, serialize, serialize_into};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct MyStruct {
    name: String,
}

#[test]
fn bincode_con() {
    let structs = vec![
        MyStruct {
            name: "A".to_string(),
        },
        MyStruct {
            name: "Hello".to_string(),
        },
        MyStruct {
            name: "Foo".to_string(),
        },
        MyStruct {
            name: "".to_string(),
        },
        MyStruct {
            name: "Test".to_string(),
        },
    ];

    let mut encoded: Vec<u8> = Vec::new();
    let x = serialize(&structs).unwrap();
    println!("X {:?}", x);

    for x in &structs {
        let pos = encoded.len();
        encoded.write(&[0]).expect("Couldn't write.");
        serialize_into(&mut encoded, x).expect("Couldn't serialize.");
        let len = encoded.len() - pos;
        encoded[pos] = len as u8;
    }

    println!("XX {:?}", encoded);

    let mut actual: Vec<MyStruct> = Vec::new();
    let mut index = 0;
    loop {
        if index >= encoded.len() {
            break;
        }
        let x: MyStruct = deserialize(&encoded[(index + 1)..]).unwrap();
        actual.push(x);
        index += encoded[index] as usize;
    }

    assert_eq!(structs, actual);
}
