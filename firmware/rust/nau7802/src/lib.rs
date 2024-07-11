
pub mod registers;
pub mod nau7802;

#[cfg(test)]
mod tests {
    use super::*;

    use embedded_hal_mock::eh1::i2c::*;
}
