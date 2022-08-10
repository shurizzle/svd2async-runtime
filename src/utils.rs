pub trait InPlace {
    type Borrowed: ToOwned<Owned = Self> + ?Sized;

    fn in_place<F: FnOnce(&Self::Borrowed) -> &Self::Borrowed>(&mut self, mutator: F);
}

impl InPlace for String {
    type Borrowed = str;

    fn in_place<F: FnOnce(&str) -> &str>(&mut self, mutator: F) {
        let (start, len): (*const u8, usize) = {
            let self_mutated: &str = mutator(self);
            (self_mutated.as_ptr(), self_mutated.len())
        };
        unsafe { ::core::ptr::copy(start, self.as_bytes_mut().as_mut_ptr(), len) };
        self.truncate(len);
    }
}

pub trait StringTrimInPlace {
    fn trim_in_place(&mut self);
}

pub fn starts_with_case_insensitive(a: &str, b: &str) -> bool {
    if b.len() > a.len() {
        return false;
    }

    a.chars()
        .flat_map(char::to_lowercase)
        .zip(b.chars().flat_map(char::to_lowercase))
        .all(|(a, b)| a == b)
}
