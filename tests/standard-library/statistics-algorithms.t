use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
median is std statistics median
modes is std statistics modes
histogram is std statistics histogram
population-variance is std statistics population-variance
sample-variance is std statistics sample-variance
quantile is std statistics quantile
covariance is std statistics covariance
summarize is std statistics summarize
summary-add is std statistics summary-add
summary-merge is std statistics summary-merge
summary-mean is std statistics summary-mean
summary-population-variance is std statistics summary-population-variance

four : List Int is Entry (1, Entry (2, Entry (3, Entry (4, Empty))))
repeated : List Int is Entry (3, Entry (1, Entry (3, Entry (2, Entry (1, Empty)))))
none : List Int is Empty
one-value : List Int is one 7
expected-modes : List Int is Entry (3, Entry (1, Empty))
two-and-half : Optional Rational is Some (Rational (5, 2))
five-fourths : Optional Rational is Some (Rational (5, 4))
five-thirds : Optional Rational is Some (Rational (5, 3))
no-rational : Optional Rational is None Rational

exact-median : Pass is Pass ((median four) = two-and-half)
empty-median : Pass is Pass ((median none) = no-rational)
stable-modes : Pass is Pass ((modes repeated) = expected-modes)
histogram-size : Pass is Pass ((entry-count (histogram repeated)) = 3)
population-exact : Pass is Pass ((population-variance four) = five-fourths)
sample-exact : Pass is Pass ((sample-variance four) = five-thirds)
sample-undersized : Pass is Pass ((sample-variance one-value) = no-rational)
middle-quantile : Pass is Pass ((quantile (four, Rational (1, 2))) = two-and-half)
self-covariance : Pass is Pass ((covariance (four, four)) = five-fourths)
stream-mean : Pass is Pass ((summary-mean (summarize four)) = two-and-half)
stream-variance : Pass is Pass ((summary-population-variance (summarize four)) = five-fourths)
extended-summary is summary-add (summarize four, Rational 5)
extended-mean : Pass is Pass ((summary-mean extended-summary) = (Some (Rational 3)))
first-half : List Int is Entry (1, Entry (2, Empty))
second-half : List Int is Entry (3, Entry (4, Empty))
merged-summary is summary-merge (summarize first-half, summarize second-half)
merged-mean : Pass is Pass ((summary-mean merged-summary) = two-and-half)

(exact-median, empty-median, stable-modes, histogram-size, population-exact,
 sample-exact, sample-undersized, middle-quantile, self-covariance,
 stream-mean, stream-variance, extended-mean, merged-mean)
