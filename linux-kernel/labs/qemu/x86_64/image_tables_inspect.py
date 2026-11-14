"""Observe owned inactive table construction through the shared read-only harness."""
from image_backing_inspect import main

if __name__ == '__main__':
    main(tables=True)
