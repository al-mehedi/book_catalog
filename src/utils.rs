use crate::*;

/*
  Get readtime from user input
*/
pub fn readtime() -> Option<u32> {
  let mut user_input = String::new();
  println!("Enter your book time in hour (e.g., 5) :");
  let _ = std::io::stdin().read_line(&mut user_input).unwrap();
  
  
  if let Ok(hours) = user_input.trim().parse::<u32>() {
    if hours == 0  {
      println!("User input must be a grater than 0!");
      None
    } else {
      Some(hours * 60) // Converts user input into minutes
    }
  } else {
    println!("Invalid user input!");
    None
  }
}

/*
  Get holiday recommendation date from user input
*/
pub fn recommendation() -> String {
  let mut user_input = String::new();
  println!("Enter your holiday recommendation date (e.g., 1701) :");
  let _ = std::io::stdin().read_line(&mut user_input).unwrap();
  
  user_input.trim().to_string()
}

/*
  Read all the files from a given directory
  and returns a vector of strings containing all file contents
*/
pub fn read_file_contents(dir: &str, reg_ex: Regex) -> Vec<String> {
  let mut contents = Vec::new();

  let dir_path = format!("Unable to find {dir} directory");
  let paths = fs::read_dir(dir).expect(&dir_path);

  for path in paths {
    let filepath = path.unwrap().path();

    let path_str = filepath.to_str().unwrap();
    if reg_ex.is_match(path_str) {
      let data = fs::read_to_string(filepath).unwrap();
      contents.push(data);
    }
  }

  contents
}

/*
  Read all the files from a given directory
  and returns a vector of strings containing all file contents
*/
pub fn read_holiday_file_contents(dir: &str, file: &str) -> String {
  let filepath = format!("{}/{}.xml", dir, file);
  fs::read_to_string(filepath).unwrap()
}


/*
  Returns unique items from given vectors
*/
pub fn u_items (mut items: Vec<String>, to_remove: Vec<String>) -> Vec<String> {
  let to_remove = BTreeSet::from_iter(to_remove);
  items.retain(|e| !to_remove.contains(e));
  items
}

/*
  Extracts Book Id
*/
pub fn extract_id(book: &str) -> Option<String> {
  let regex = Regex::new(r#"id=".*?""#).unwrap();
  if let Some(book_id) = regex.find(&book) {
    let book_id = book_id.as_str();
    Some(book_id[4..book_id.len() - 1].to_owned())
  } else {
    None
  }
}

/*
  Extracts Multiple Book Id
*/
pub fn extract_ids(library: &str) -> Vec<String> {
  let regex = Regex::new(r#"\sid=".*?""#).unwrap();

  let mut ids = Vec::new();
  for book_id in regex.find_iter(library) {
    let id = book_id.as_str();
    if !id.contains("lib") {
      ids.push(id[5..id.len() - 1].to_owned())
    }
  }

  ids
}

/*
  Extracts Readtime from Book
*/
pub fn extract_readtime(book: &str) -> Option<String> {
  let regex = Regex::new(r#"readtime["=>]+[[:digit:]]+[=<]?"#).unwrap();
  if let Some(readtime) = regex.find(&book) {
    let readtime = readtime.as_str();
    if readtime.contains("=") {
      Some(readtime[10..readtime.len()].to_owned())
    } else {
      Some(readtime[9..readtime.len()].to_owned())
    }
  } else {
    None
  }
}

pub fn lib_time(start_time: &str, end_time: &str) -> u32 {
  let start_at = Library::time_in_minutes(start_time);
  let mut end_at = Library::time_in_minutes(end_time);

  // 1440 is the number of minutes in a day
  if end_at < start_at { end_at += 1440; }

  let st = offset::Local::now().hour();
  let stm = offset::Local::now().minute();
  let start_at = (st * 60) + stm;

  end_at - start_at
}