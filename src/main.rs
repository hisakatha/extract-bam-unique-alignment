fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rust_htslib::{bam, bam::Read};
    use std::convert::TryInto;
    let args: Vec<String> = std::env::args().collect();
    if args.len() - 1 != 3 {
        let mut usage = String::from("#arguments must be 3");
        usage.push_str("\nArg1: BAM file sorted/grouped by name (unsorted file will be used without warnings)");
        usage.push_str("\nArg2: Map Qv (score) threshold (inclusive, integer)");
        usage.push_str("\nArg3: output BAM file name");
        eprintln!("{}", usage);
        panic!("#arguments must be 3. observed: {}", args.len() - 1);
    }
    // Assuming the input bam is sorted by template name
    //let bam_path = &"mapped.alignmentset.merged.sorted_by_name.bam";
    let bam_path = &args[1];
    let mapq_threshold_raw: i32 = args[2].parse().unwrap();
    if mapq_threshold_raw < 0 || mapq_threshold_raw > 255 {
        panic!("Map Qv threshold must be an integer from 0 to 255");
    }
    let mapq_threshold: u8 = mapq_threshold_raw.try_into().unwrap();
    let mut bam = bam::Reader::from_path(bam_path)?;
    let out_bam_path = &args[3];
    let header = bam.header();
    let out_bam_header = bam::Header::from_template(header);
    // TODO: add a header record of this program. Need a unique ID.
    //let mut out_bam_header = bam::Header::from_template(header);
    //use rust_htslib::{bam::header::Header, bam::header::HeaderRecord};
    //out_bam_header.push_record(Header::new(b"PG").push_record());
    let mut out_bam = bam::Writer::from_path(out_bam_path, &out_bam_header, bam::Format::BAM).unwrap();
    let num_tid: i32 = header.target_count().try_into().unwrap();
    eprintln!("INFO: target_count: {}", num_tid);
    (0 .. num_tid).for_each(|x| eprintln!("INFO: tid {} = {:?}", x, std::str::from_utf8(header.tid2name(x.try_into().unwrap())).unwrap()));
    let mut record = bam::Record::new();
    let mut last_record = record.clone();
    let mut is_unique_alignment = false;
    let mut num_valid: u64 = 0;
    let mut num_skipped_unmapped = 0;
    let mut num_skipped_low_mapq = 0;
    let mut current_qname: String;
    let mut last_qname: String = "".to_string();
    let mut current_tid;
    let mut current_mapq: u8;
    while let Some(result) = bam.read(&mut record) {
        result.expect("Failed to parse a record");
        current_qname = std::str::from_utf8(record.qname())?.to_string();
        current_tid = record.tid();
        current_mapq = record.mapq();
        if current_tid == -1 || current_mapq == 255 {
            num_skipped_unmapped += 1;
        } else if current_tid < num_tid {
            if current_mapq >= mapq_threshold {
                if current_qname != last_qname {
                    if is_unique_alignment {
                        out_bam.write(&last_record)?;
                    }
                    is_unique_alignment = true;
                    last_record = record.clone();
                    last_qname = current_qname.clone();
                } else {
                    is_unique_alignment = false;
                }
                num_valid += 1;
            } else {
                num_skipped_low_mapq += 1;
            }
        } else {
            panic!("Unexpected tid: {}", current_tid);
        }
        //println!("QNAME: {:?}, TID: {:?}, MAPQ: {:?}", current_qname, current_tid, current_mapq);
    }
    // Write the result for the last block
    if is_unique_alignment {
        out_bam.write(&last_record)?;
    }
    eprintln!("INFO: # valid alignments: {}", num_valid);
    eprintln!("INFO: # unmapped reads: {}", num_skipped_unmapped);
    eprintln!("INFO: # low MapQ alignments: {}", num_skipped_low_mapq);
    Ok(())
}
