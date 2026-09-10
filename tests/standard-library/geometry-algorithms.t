use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
nearest-component-product is std geometry nearest-component-product
final-connection-x-product is std geometry final-connection-x-product
largest-point-rectangle is std geometry largest-point-rectangle
largest-contained-rectangle is std geometry largest-contained-rectangle

points3 : List (Int, Int, Int) is Entry (
  (0, 0, 0),
  Entry ((1, 0, 0), Entry ((10, 0, 0), Entry ((11, 0, 0), Empty)))
)
corners : List (Int, Int) is Entry ((0, 0), Entry ((2, 2), Empty))
polygon : List (Int, Int) is Entry (
  (0, 0),
  Entry ((2, 0), Entry ((2, 2), Entry ((0, 2), Empty)))
)

nearest-components : Pass is Pass ((nearest-component-product (points3, 1)) = 2)
final-connection : Pass is Pass ((final-connection-x-product points3) = 10)
point-rectangle : Pass is Pass ((largest-point-rectangle corners) = 9)
contained-rectangle : Pass is Pass ((largest-contained-rectangle polygon) = 9)

(nearest-components, final-connection, point-rectangle, contained-rectangle)
