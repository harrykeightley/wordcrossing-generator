# Wordcrossing Level Generation
This program generates levels for the online game [wordcrossing.com](https://wordcrossing.com).

I ported this to rust since the original typescript implementation was getting cut off by 
cloudflare workflow timeouts and causing missing levels on the site. 

## Report
A report on the level generation process is listed at `report/wc.pdf`. 
I apologise in advance for the poor formatting of algorithms.

## Configuration
The parameters for:
- Number of levels to generate
- Level dimensions
- And some others, only relevant for formatting the levels as I would need for storing 
  them in the bucket.
are configured through the constants inside `src/main.rs`. 
A future extension of this would be to add some simple 
command line argument parsing.

## Output
When run, the program visualises the requested number of levels in the output.
If instead, you would like to save the levels to disk, you can uncomment the code 
at the end of `main.rs:main`, which creates the requested number of 
levels in the `assets/output` folder. 
You will have to mkdir this directory if it doesn't already exist to run this without 
panicking.
