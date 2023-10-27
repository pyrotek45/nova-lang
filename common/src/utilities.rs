pub fn is_string_number(data: &str) -> bool {
    let mut deci = false;
    let mut start_index = 0;
    if data.is_empty() {
        return false;
    }
    if data.starts_with('-') {
        start_index = 1;
        // Check that there is at least one numeric or '.' character after the '-' symbol
        if data.len() == 1
            || (data.len() > 1 && !data.chars().skip(1).any(|c| c.is_numeric() || c == '.'))
        {
            return false;
        }
    }
    if data[start_index..].starts_with('.') {
        return false;
    }
    for (i, c) in data.chars().enumerate().skip(start_index) {
        //Checks to see if there is more than one period
        if c == '.' && deci {
            return false;
        }
        //Checks to see if it is a number, and makes sure it skips first period
        if !c.is_numeric() && c != '.' {
            return false;
        }
        //Changes deci to true after finding first period
        if c == '.' {
            deci = true
        }
        // Allows '-' symbol only at the beginning of the string
        if c == '-' && i != start_index {
            return false;
        }
    }
    true
}
