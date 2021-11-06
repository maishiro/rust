extern crate plotly;
use std::fs::File;
use plotly::common::Mode;
use plotly::{Plot, Scatter, ImageFormat};
use encoding_rs;
use std::fs;
use regex::Regex;
use std::time::Instant;
use polars::prelude::*;
use polars::df;

fn main() {

    let s = fs::read("logfile.log").unwrap();

    // SHIFT_JISのバイト列(Vec<u8>) を UTF-8の文字列(String) に変換
    let (res, _, _) = encoding_rs::SHIFT_JIS.decode(&s);
    let lines = res.into_owned();


    let vec_regs = vec![    ( (Regex::new(r".*Func1 Start").unwrap(), vec!["main","Func1"] ) ),
                            ( (Regex::new(r".*Func1 End").unwrap(), vec!["Func1","main"] ) ),
                        ];
    let reg_date_time = Regex::new(r"\[(\d{4})/(\d{2})/(\d{2}) (\d{2}:\d{2}:\d{2}):(\d{3})\](.*)").unwrap();    


    let mut vec_datetime: Vec<String> = Vec::new();
    let mut vec_method: Vec<String> = Vec::new();
    let mut vec_message: Vec<String> = Vec::new();


    let start = Instant::now();

    let vv: Vec<&str> = lines.split("\r\n").collect();
    for line in vv {
        for regs in vec_regs.iter() {
            match regs.0.captures(line) {
                Some(_) => {
                    match reg_date_time.captures(line) {
                        Some(vals) => {
                            let str_dateime = format!("{}-{}-{} {}.{}", &vals[1], &vals[2], &vals[3], &vals[4], &vals[5] );

                            for k in regs.1.iter() {
                                vec_datetime.push( str_dateime.clone() );
                                vec_method.push( k.to_string() );
                                vec_message.push( format!("{}", &vals[6]) );
                            }
                        },
                        _ => ()
                    }

                    break;
                },
                _ => ()
            }
        }
    }

    let end = start.elapsed();
    println!( "elapsed time {}.{:03} [sec]", end.as_secs(), end.subsec_nanos() / 1_000_000 );


    let s0 = Series::new( "DATETIME", vec_datetime.clone() );
    let s1 = Series::new( "METHOD", vec_method.clone() );
    let s2 = Series::new( "MESSAGE", vec_message.clone() );
    let df = DataFrame::new( vec![s0, s1,s2] ).unwrap();

    // create a file
    let mut file = File::create("data.csv").expect("could not create file");

    // write DataFrame to file
    CsvWriter::new( &mut file )
        .has_header( true )
        .with_delimiter( b',' )
        .finish( &df );


    let trace1 = Scatter::new( vec_datetime, vec_method )
        .text_array( vec_message )
        .name("trace1")
        .mode( Mode::LinesMarkers );

    let mut plot = Plot::new();
    plot.add_trace(trace1);
    // plot.show();

    plot.to_html( "graph.html" );

    // plot.show_png( 1280, 900 );
    // plot.save( "graph.png", ImageFormat::PNG, 1280, 900, 1.0 );

}
