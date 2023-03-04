use crate::*;
use crate::Utils;

/* Structure for Book */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book { pub id: String, pub readtime: u32, pub content: String }

impl Book {
  // Converts readtime from ms to minute
  pub fn read_time(&self) -> u32 {
    let minutes = (self.readtime / 1000) / 60;
    minutes
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct BookMetaData { id: String, readtime: u32 }

/*
  Get only generic books from file system
  and store as a string into a hash_map with book_id
*/
pub fn get_generic_books(dir: &str) -> HashMap<String, Books::Book>{
  let mut hash_map: HashMap<String, Books::Book> = HashMap::new();

  /*
    Incase `dir` contains files other than books
    This was implemented for earlier version.
  */
  let pattern = Regex::new(r".xml$").unwrap();

  let books = Utils::read_file_contents(dir, pattern);

  for book in books {
    let book_id = Utils::extract_id(&book).unwrap();
    let readtime = Utils::extract_readtime(&book).unwrap_or("0".to_owned());

    let data = Book {
      id: book_id.clone(),
      readtime: readtime.parse::<u32>().unwrap(),
      content: book
    };

    // Assumes all books are unique
    hash_map.insert(book_id, data);
  }

  hash_map
}