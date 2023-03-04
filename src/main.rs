use std::fs;
use rand::Rng;
use serde_xml_rs;
use regex::Regex;
use chrono::offset;
use chrono::prelude::*;
use std::collections::{ BTreeSet, HashMap };
use serde_derive::{ Serialize, Deserialize };

/* 
  Test data directories
  Should be changed here if directory name changes
*/
const DIR_BOOK: &str = "files";
const DIR_LIBRARY: &str = "library";
const DIR_HOLIDAY: &str = "holiday";

/*
  Repeat frequency for general books
  Limit on how ofter a same book can repeat on the catalog

  IMPORTANT: Limit must be less than minimum number of Book IDs in lib*.xml
*/
const REPEAT_AFTER: usize = 4;

mod utils;
use utils as Utils;

mod books;
use books as Books;

mod library;
use library as Library;

mod holiday;
use holiday as Holiday;


fn main() {
  gen_catalog();
  // gen_non_repeated_catalog();
  // gen_non_repeated_catalog_with_holiday_recommendation();
}


/*
  Generate catalogs based on default criteria 
*/
fn gen_catalog() {
  let mem_books = Books::get_generic_books(DIR_BOOK);
  let mem_libraries = Library::get_generic_libraries(DIR_LIBRARY);

  loop {
    let user_readtime = if let Some(minutes) = Utils::readtime() { minutes } else { continue; };

    if let Some(libraries) = Library::get_available_libraries(&mem_libraries) {
      println!("{} libraries found!", libraries.len());
      // println!("{} libraries found!\n{:?}", libraries.len(), libraries);

      let mut total_readtime = 0;
      let mut book_list = Vec::new();
      let mut catalog_items = Vec::new();
      let mut catalog = String::from("<?xml version=\"1.0\"?>\n<catalog-list>\n");

      for library in libraries {
        // Check guard for early breakout
        if total_readtime >= user_readtime { break; }

        // Calculate a library readtime based on current time
        let st = library.metadata.starttime.as_str();
        let et = library.metadata.endtime.as_str();
        let lib_time = Utils::lib_time(st, et);

        let mut session = 0;
        loop {
          if session < lib_time && total_readtime < user_readtime {
            let book_id = library.get_rand_book(&mut book_list);
            let book = mem_books.get(&book_id).unwrap();

            if book.read_time() > 0 {
              session += book.read_time();
              total_readtime += book.read_time();
              catalog_items.push(book.content.to_owned());
            }

            continue;
          }

          break;
        }
      }
     
      let books = catalog_items.join("");
      catalog.push_str(&format!("{books}</catalog-list>"));

      println!("Saving catalog.xml in working directory\n\n");
      fs::write("catalog.xml", catalog).expect("Unable to write file");
    } else {
      println!("No libraries are open at the moment!\n");
    }
  }
}


/*
  Generate catalogs based on default criteria
  With non repeated book sequence
*/
fn gen_non_repeated_catalog() {
  let mem_books = Books::get_generic_books(DIR_BOOK);
  let mem_libraries = Library::get_generic_libraries(DIR_LIBRARY);

  loop {
    let user_readtime = if let Some(minutes) = Utils::readtime() { minutes } else { continue; };

    if let Some(libraries) = Library::get_available_libraries(&mem_libraries) {
      println!("{} libraries found!", libraries.len());
      // println!("{} libraries found!\n{:?}", libraries.len(), libraries);

      let mut total_readtime = 0;
      let mut book_list = Vec::new();
      let mut catalog_items = Vec::new();
      let mut catalog = String::from("<?xml version=\"1.0\"?>\n<catalog-list>\n");

      for library in libraries {
        // Check guard for early breakout
        if total_readtime >= user_readtime { break; }

        // Calculate a library readtime based on current time
        let st = library.metadata.starttime.as_str();
        let et = library.metadata.endtime.as_str();
        let lib_time = Utils::lib_time(st, et);

        let mut session = 0;
        loop {
          if session < lib_time && total_readtime < user_readtime {
            let book_id = library.get_non_repeated_book(&mut book_list);
            let book = mem_books.get(&book_id).unwrap();

            if book.read_time() > 0 {
              session += book.read_time();
              total_readtime += book.read_time();
              catalog_items.push(book.content.to_owned());
            }

            continue;
          }

          break;
        }
      }
     
      let books = catalog_items.join("");
      catalog.push_str(&format!("{books}</catalog-list>"));

      println!("Saving catalog.xml in working directory\n\n");
      fs::write("catalog.xml", catalog).expect("Unable to write file");
    } else {
      println!("No libraries are open at the moment!\n");
    }
  }
}

/*
  Generate catalogs based on default criteria
  With non repeated book sequence and holiday recommendation
*/
fn gen_non_repeated_catalog_with_holiday_recommendation() {
  let mem_books = Books::get_generic_books(DIR_BOOK);
  let mem_libraries = Library::get_generic_libraries(DIR_LIBRARY);
  let mem_holiday = Holiday::get_holiday_library(DIR_HOLIDAY, "holiday.xml");

  loop {
    let user_readtime = if let Some(minutes) = Utils::readtime() { minutes } else { continue; };

    let user_recommendation = Utils::recommendation();


    if let Some(libraries) = Library::get_available_libraries(&mem_libraries) {
      println!("{} libraries found!", libraries.len());
      // println!("{} libraries found!\n{:?}", libraries.len(), libraries);

      let mut skip_readtime = 0;
      let mut total_readtime = 0;
      let mut book_list = Vec::new();
      let mut catalog_items = Vec::new();
      let mut catalog = String::from("<?xml version=\"1.0\"?>\n<catalog-list>\n");

      /* Checks if holiday book is recommendable */
      let frequencies = &mem_holiday.holiday_lib.frequencies.frequency;
      let repeat_frequency = Holiday::is_recommendable(&user_recommendation, frequencies);

      for library in libraries {
        // Check guard for early breakout
        if catalog_items.len() > 0 {
          if total_readtime - skip_readtime >= user_readtime { break; }
        } else {
          if total_readtime >= user_readtime { break; }
        }

        // Calculate a library readtime based on current time
        let st = library.metadata.starttime.as_str();
        let et = library.metadata.endtime.as_str();
        let lib_time = Utils::lib_time(st, et);

        let mut session = 0;
        loop {
          if session < lib_time
          && total_readtime - skip_readtime < user_readtime {
            if let Some(rf) = repeat_frequency {
              if book_list.len() % (rf + 1) == 0 {

                /* 
                  No Holiday Books
                  So generates random book from books
                */
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..mem_books.len());
                let books: Vec<Books::Book> = mem_books.values().cloned().collect();
                
                let book_id = books[index].id.to_owned();
                if books[index].read_time() > 0 {
                  book_list.push(book_id.to_owned());

                  session += books[index].read_time();
                  total_readtime += books[index].read_time();

                  /* Mimics Holiday Book */
                  catalog_items.push(format!("<holiday>FOLLOWING BOOK IS HOLIDAY BOOK</holiday>{}", &books[index].content.to_owned()));

                  if skip_readtime == 0 {
                    skip_readtime = books[index].read_time();
                  }
                }

                continue;
              }
            }

            let book_id = library.get_non_repeated_book(&mut book_list);
            let book = mem_books.get(&book_id).unwrap();

            session += book.read_time();
            total_readtime += book.read_time();
            catalog_items.push(book.content.to_owned());

            continue;
          }

          break;
        }
      }
     
      let books = if let Some(_) = repeat_frequency {
        catalog_items[1..].join("")
      } else {
        catalog_items.join("")
      };
      
      catalog.push_str(&format!("{books}</catalog-list>"));

      println!("Saving catalog.xml in working directory\n\n");
      fs::write("catalog.xml", catalog).expect("Unable to write file");
    } else {
      println!("No libraries are open at the moment!\n");
    }
  }
}