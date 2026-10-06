#![cfg(feature = "1_0")]

// make sure attribute-list validation doesn't permit OOB reads.

use khronos_egl::{
    check_attrib_list, check_int_list, Attrib, Error, Int, GREEN_SIZE, NONE, RED_SIZE,
    TRANSPARENT_TYPE,
};

const CASES: &[(&[Int], Result<(), Error>)] = &[
    (&[], Err(Error::BadParameter)),
    (&[RED_SIZE], Err(Error::BadParameter)),
    (&[RED_SIZE, 8], Err(Error::BadParameter)),
    (&[RED_SIZE, NONE], Err(Error::BadParameter)),
    (&[RED_SIZE, 8, GREEN_SIZE, NONE], Err(Error::BadParameter)),
    (&[RED_SIZE, NONE, GREEN_SIZE], Err(Error::BadParameter)),
    (
        &[RED_SIZE, NONE, GREEN_SIZE, NONE],
        Err(Error::BadParameter),
    ),
    (&[NONE], Ok(())),
    (&[RED_SIZE, 8, NONE], Ok(())),
    (&[RED_SIZE, 8, GREEN_SIZE, 8, NONE], Ok(())),
    (&[TRANSPARENT_TYPE, NONE, NONE], Ok(())),
    (&[NONE, NONE], Ok(())),
    (&[RED_SIZE, 8, NONE, NONE], Ok(())),
    (&[RED_SIZE, 8, NONE, GREEN_SIZE], Ok(())),
];

#[test]
fn int_lists_require_a_terminator_in_an_attribute_position() {
    for &(list, expected) in CASES {
        assert_eq!(check_int_list(list), expected, "list: {list:?}");
    }
}

#[test]
fn attrib_lists_require_a_terminator_in_an_attribute_position() {
    for &(list, expected) in CASES {
        let list: Vec<Attrib> = list
            .iter()
            .map(|&value| Attrib::try_from(value).unwrap())
            .collect();
        assert_eq!(check_attrib_list(&list), expected, "list: {list:?}");
    }
}
