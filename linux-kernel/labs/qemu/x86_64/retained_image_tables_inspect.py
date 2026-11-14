"""Run the scoped-retention protocol in the authorized bounded VM."""
from image_backing_inspect import main

if __name__=='__main__': main(tables=True,retained=True)
