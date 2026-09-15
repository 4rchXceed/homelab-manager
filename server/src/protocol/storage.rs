pub struct Storage {
    pub id: String,
    pub path: String,
    pub fallback: String,
    pub do_not_create: bool,
    pub is_invalid: bool,
}

impl Storage {
    pub fn new(
        id: String,
        path: String,
        is_invalid: bool,
        fallback: String,
        do_not_create: bool,
    ) -> Self {
        return Storage {
            id: id,
            path: path,
            fallback: fallback,
            do_not_create: do_not_create,
            is_invalid: is_invalid,
        };
    }
}
