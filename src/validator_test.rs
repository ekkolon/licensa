use super::*;

#[test]
fn valid_license_year() {
  assert_eq!(acceptable_year("1956"), Ok(1956));
}

#[test]
fn valid_earliest_license_year() {
  assert_eq!(
    acceptable_year(&EARLIEST_LICENSE_YEAR.to_string()),
    Ok(EARLIEST_LICENSE_YEAR)
  );
}

#[test]
fn valid_current_year() {
  let current_year = current_year();
  assert_eq!(acceptable_year(&current_year.to_string()), Ok(current_year));
}

#[test]
fn invalid_non_numeric_input() {
  assert_eq!(
    acceptable_year("invalid"),
    Err("`invalid` isn't a valid year".to_string())
  );
}

#[test]
fn invalid_future_year() {
  let this_year = current_year();
  let future_year = this_year + 1;
  let actual = acceptable_year(&future_year.to_string());
  let expected = Err(format!(
    "Year not in valid range {}-{}",
    EARLIEST_LICENSE_YEAR, this_year
  ));

  assert_eq!(actual, expected);
}

#[test]
fn invalid_earlier_than_earliest_license_year() {
  let earlier_year = EARLIEST_LICENSE_YEAR - 1;
  let actual = acceptable_year(&earlier_year.to_string());
  let expected = Err(format!(
    "Year not in valid range {}-{}",
    EARLIEST_LICENSE_YEAR,
    current_year()
  ));

  assert_eq!(actual, expected);
}
