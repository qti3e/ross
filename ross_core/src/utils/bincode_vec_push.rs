use bincode::{deserialize, serialize_into};

/// Push number of new items that are already serialized into an optional already-serialized vector
/// of elements (`existing`), if the first value is not provided a new vector with `len=0` is
/// created and used instead.
/// This function is used in `DB` as our RocksDB's merge operator for values of type `Vec`.
#[inline]
pub fn merge_push<'a>(existing: Option<&[u8]>, items: impl Iterator<Item = &'a [u8]>) -> Vec<u8> {
    let mut first = true;
    let est_count = items.size_hint().0;

    let (mut count, mut result): (u64, Vec<u8>) = match existing {
        Some(bytes) => {
            let c = deserialize::<u64>(bytes).unwrap();
            (c, bytes.into())
        }
        None => (0, [0; 8].into()),
    };

    for buf in items {
        if first {
            first = false;
            let size = est_count * buf.len();
            result.reserve_exact(size);
        }

        count += 1;
        result.extend_from_slice(buf);
    }

    serialize_into(&mut result[0..8], &count).unwrap();

    result
}

#[cfg(test)]
mod test {
    use super::merge_push;
    use bincode::{deserialize, serialize};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
    struct Item(i32, i32);

    fn run_test(existing: Option<Vec<Item>>, mut items: Vec<Item>) {
        let items_vec = {
            let mut result = Vec::with_capacity(items.len());
            for item in &items {
                let v = serialize(item).unwrap();
                result.push(v);
            }
            result
        };

        let items_serialized = {
            let mut result = Vec::with_capacity(items.len());
            for item in &items_vec {
                result.push(&**item)
            }
            result
        };

        let existing_serialized = existing.clone().map(|v| serialize(&v).unwrap());
        let result_serialized =
            merge_push(existing_serialized.as_deref(), items_serialized.into_iter());
        let result_decoded = deserialize::<Vec<Item>>(&result_serialized).unwrap();

        let mut result = Vec::new();
        if let Some(mut e) = existing {
            result.append(&mut e);
        }
        result.append(&mut items);

        assert_eq!(result_decoded, result);
    }

    #[test]
    fn test() {
        run_test(None, Vec::new());
        run_test(None, vec![Item(0, 0)]);
        run_test(None, vec![Item(17, 9), Item(5, 27)]);
        run_test(Some(vec![Item(17, 9), Item(5, 27)]), vec![]);
        run_test(Some(vec![Item(17, 9), Item(5, 27)]), vec![Item(12, 13)]);
        run_test(
            Some(vec![Item(17, 9), Item(5, 27)]),
            vec![Item(12, 13), Item(8, 7)],
        );
    }
}
