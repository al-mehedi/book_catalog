use std::fs;
use rand::Rng;
use serde_xml_rs;
use regex::Regex;
use chrono::offset;
use chrono::prelude::*;
use std::collections::{ BTreeSet, HashMap };
use serde_derive::{ Serialize, Deserialize };


const DIR_BOOK: &str = "files";
const DIR_LIBRARY: &str = "library";

/*
  Limit on how ofter a same book can repeat on the catalog
*/
const REPEAT_AFTER: usize = 5;


/* Structure for Book */

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Book { id: String, readtime: u32, content: String }

impl Book {
  // Converts readtime from ms to minute
  fn read_time(&self) -> u32 {
    let minutes = (self.readtime / 1000) / 60;
    minutes
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct BookMetaData { id: String, readtime: u32 }


/* Structure for Library */

#[derive(Debug, Serialize, Deserialize)]
struct BookId { id: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LibraryMetadata { starttime: String, endtime: String, opendays: u32 }

#[derive(Debug, Serialize, Deserialize)]
struct Catalog { book: Vec<BookId>, library: LibraryMetadata }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Library { books: Vec<String>, metadata: LibraryMetadata }

impl Library {
  fn get_non_repeated_book(&self, book_list: &mut Vec<String>) -> String {
    let mut rng = rand::thread_rng();

    if book_list.len() < 6 {
      let to_remove = book_list.clone();
      let unique_books = u_items(self.books.clone(), to_remove);
      let book_id = &unique_books[rng.gen_range(0..unique_books.len())];
      book_list.push(book_id.to_owned());
      book_id.to_owned()
    } else {
      let offset = (book_list.len() / REPEAT_AFTER) * REPEAT_AFTER;
      let to_remove = Vec::from_iter(book_list[offset..].iter().cloned());
      let unique_books = u_items(self.books.clone(), to_remove);
      let book_id = &unique_books[rng.gen_range(0..unique_books.len())];
      book_list.push(book_id.to_owned());
      book_id.to_owned()
    }
  }
}

/* Holiday Library Structures */

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HolidayLibrary {
  #[serde(rename = "holiday-lib")]
  holiday_lib: HolidayLibTag
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HolidayLibTag { uid: String, frequencies: Frequency }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Frequency { frequency: Vec<FrequencyAttr> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrequencyAttr { recommended: String, date: String }


fn main() {
  /* BOOK STORAGE */
  let mut storage_books: HashMap<String, Book> = HashMap::new();
  let filename = Regex::new(r"bk\d+.xml$").unwrap();
  get_generic_books(DIR_BOOK, filename , &mut storage_books);

  // println!("************* GENERIC BOOKS  *************");
  // println!("{storage_books:#?}");

  /* LIBRARY STORAGE */
  let mut storage_libraries: Vec<Library> = Vec::new();
  let filename = Regex::new(r"lib\d+.xml$").unwrap();
  get_generic_libraries(DIR_LIBRARY, filename, &mut storage_libraries);

  // println!("************* GENERIC LIBRARIES *************");
  // println!("{storage_libraries:#?}");

  let lib_holiday = get_holiday_library(DIR_LIBRARY, "holiday.xml");
  // println!("************* HOLIDAY LIBRARY *************");
  // println!("{lib_holiday:#?}");

  let mut storage_holiday_books: HashMap<String, Book> = HashMap::new();
  get_holiday_books(DIR_LIBRARY, DIR_BOOK, &lib_holiday, &mut storage_holiday_books);
  // println!("************* HOLIDAY LIBRARY *************");
  // println!("{storage_holiday_books:#?}");


  loop {
    /* GET USER READTIME */
    let mut user_input = String::new();
    println!("Enter your book time in hour (e.g., 5) :");
    let _ = std::io::stdin().read_line(&mut user_input).unwrap();
    
    // Converts user input into minutes
    let input_time = if let Ok(hours) = user_input.trim().parse::<u32>() {
      if hours == 0  {
        println!("User input must be a grater than 0!");
        continue;
      }
      hours * 60
    } else {
      println!("Invalid user input!");
      continue;
    };

    /* GET USER HOLIDAY RECOMMENDATION */
    let mut user_input = String::new();
    println!("Enter your holiday recommendation date (e.g., 1701) :");
    let _ = std::io::stdin().read_line(&mut user_input).unwrap();
    
    // Take user input as holiday recommendation date
    let input_date = user_input.trim();

    /* USER QUERY STARTS HERE */
    if let Some(libraries) = get_available_libraries(&storage_libraries) {
      println!("{} libraries found!", libraries.len());

      let mut skip_readtime = 0;
      let mut total_readtime = 0;
      let mut book_list = Vec::new();
      let mut catalog_items = Vec::new();
      let mut catalog = String::from("<?xml version=\"1.0\"?>\n<catalog>\n");

      /* Checks if holiday book is recommendable */
      let frequencies = &lib_holiday.holiday_lib.frequencies.frequency;
      let repeat_frequency = is_recommendable(&input_date, frequencies);

      for library in libraries {
        // Check guard for early breakout
        if catalog_items.len() > 0 {
          if total_readtime - skip_readtime >= input_time { break; }
        } else {
          if total_readtime >= input_time { break; }
        }
        

        let start_at = library_time(library.metadata.starttime.as_str());
        let end_at = library_time(library.metadata.endtime.as_str());
        let lib_time = end_at - start_at;

        let mut session = 0;

        loop {
          if session < lib_time && total_readtime - skip_readtime < input_time {
            if let Some(rf) = repeat_frequency {
              if book_list.len() % (rf + 1) == 0 {
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..storage_holiday_books.len());
                let books: Vec<Book> = storage_holiday_books.values().cloned().collect();
                
                let book_id = books[index].id.to_owned();
                book_list.push(book_id.to_owned());

                session += books[index].read_time();
                total_readtime += books[index].read_time();
                catalog_items.push(books[index].content.to_owned());

                if skip_readtime == 0 {
                  skip_readtime = books[index].read_time();
                } 

                continue;
              }
            }

            let book_id = library.get_non_repeated_book(&mut book_list);
            let book = storage_books.get(&book_id).unwrap();

            session += book.read_time();
            total_readtime += book.read_time();
            catalog_items.push(book.content.to_owned());

            continue;
          }

          break;
        }
      }
      
      let items = if let Some(_) = repeat_frequency {
        catalog_items[1..].join("")
      } else {
        catalog_items.join("")
      };

      catalog.push_str(&format!("{items}</catalog>"));

      println!("Saving catalog.xml in working directory\n\n");
      fs::write("catalog.xml", catalog).expect("Unable to write file");
    } else {
      println!("No libraries are open at the moment!")
    }
  }
}


/*
  Get only generic books from file system
  and store as a string into a hash_map with book_id
*/
fn get_generic_books(dir: &str, pattern: Regex, hash_map: &mut HashMap<String, Book>) {
  let books = read_file_contents(dir, pattern);

  for book in books {
    let item: BookMetaData = serde_xml_rs::from_str(&book).unwrap();
    let data = Book {
      id: item.id.to_owned(),
      readtime: item.readtime,
      content: book
    };

    // Assumes all books are unique
    hash_map.insert(item.id.to_owned(), data);
  }
}

/*
  Get only holiday books from file system
  and store as a structured data into a hash_map
*/
fn get_holiday_books(lib_dir: &str, file_dir: &str, holiday: &HolidayLibrary, hash_map: &mut HashMap<String, Book>) {
  let filepath = format!("{}/{}.xml", lib_dir, holiday.holiday_lib.uid);
  let data = fs::read_to_string(filepath).unwrap();
  let item: Catalog = serde_xml_rs::from_str(data.as_str()).unwrap();

  for book in item.book.iter() {
    let filepath = format!("{}/{}.xml", file_dir, book.id);
    let book = fs::read_to_string(filepath).unwrap();
    let item: BookMetaData = serde_xml_rs::from_str(book.as_str()).unwrap();
    let data = Book {
      id: item.id.to_owned(),
      readtime: item.readtime,
      content: book
    };

    // Assumes all books are unique
    hash_map.insert(item.id.to_owned(), data);
  }
}

/*
  Get only generic libraries from file system
  and store as a vector of Library struct
*/
fn get_generic_libraries(dir: &str, pattern: Regex, list: &mut Vec<Library>) {
  let libraries = read_file_contents(dir, pattern);

  for library in libraries {
    let item: Catalog = serde_xml_rs::from_str(library.as_str()).unwrap();
    
    // Store book IDs string
    let mut book_ids = Vec::new();
    for book in item.book.iter() { book_ids.push(book.id.to_owned()) }

    list.push(Library { books: book_ids, metadata: item.library })
  }
}

/* Build holiday library structure */
fn get_holiday_library(dir: &str, filename: &str) -> HolidayLibrary {
  let filepath = format!("{dir}/{filename}");
  let data = fs::read_to_string(filepath).unwrap();
  let item: HolidayLibrary = serde_xml_rs::from_str(data.as_str()).unwrap();
  item
}

/*
  Get available libraries based on user read time
  and other business logic
*/
fn get_available_libraries(libraries: &Vec<Library>) -> Option<Vec::<Library>> {
  let mut catalogs = Vec::new();
  let current_hour = offset::Local::now().hour();
  let today = offset::Local::now().weekday().num_days_from_sunday() + 1;

  // For manual query
  // let today = 5;
  // let current_hour = 10;
  
  for library in libraries {
    if library.metadata.opendays == 0 || library.metadata.opendays == today {
      // Converts time HH:MM into minutes
      let start_at = library_time(library.metadata.starttime.as_str());
      let end_at = library_time(library.metadata.endtime.as_str());
      let time_span = end_at - start_at; 

      if time_span != 0
      && current_hour * 60 >= start_at
      && current_hour * 60 <= end_at {
        catalogs.push(library.clone());
      }
    }
  }

  if catalogs.len() == 0 { None }
  else { Some(catalogs) }
}

fn library_time(time: &str) -> u32 {
  let hours = time[..2].parse::<u32>().unwrap();
  let minutes = time[2..].parse::<u32>().unwrap();
  hours * 60 + minutes
}

/* Return recommendable holiday book frequency */
fn is_recommendable(input_date: &str, frequencies: &Vec<FrequencyAttr>) -> Option<usize> {
  for frequency in frequencies {
    if input_date == &frequency.date {
      return Some(frequency.recommended.parse::<usize>().unwrap())
    }
  }

  None
}

/*
  Read all the files from a given directory
  and returns a vector of strings containing all file contents
*/
fn read_file_contents(dir: &str, reg_ex: Regex) -> Vec<String> {
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

/* Returns unique items from given vectors */
fn u_items (mut items: Vec<String>, to_remove: Vec<String>) -> Vec<String> {
  let to_remove = BTreeSet::from_iter(to_remove);
  items.retain(|e| !to_remove.contains(e));
  items
}
