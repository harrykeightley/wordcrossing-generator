# Wordcrossing Level Generation
This program generates levels for the online game [wordcrossing.com](https://wordcrossing.com).

I ported this to rust since the original typescript implementation was getting cut off by 
cloudflare workflow timeouts and causing missing levels on the site. 

## Usage
This is a no batteries included version of the program, and it's configured through the 
constaints inside `src/main.rs`. A future extension of this would be to add some simple 
command line argument parsing.

When run, the program creates the requested number of levels in the `assets/output` folder.
You will have to mkdir this directory if it doesn't already exist to run this without 
panicing.
