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
rational-four : List Rational is Entry (Rational 1, Entry (Rational 2, Entry (Rational 3, Entry (Rational 4, Empty))))
rational-repeated : List Rational is Entry (Rational 3, Entry (Rational 1, Entry (Rational 3, Entry (Rational 2, Entry (Rational 1, Empty)))))
no-rationals : List Rational is Empty
expected-modes : List Int is Entry (3, Entry (1, Empty))
expected-rational-modes : List Rational is Entry (Rational 3, Entry (Rational 1, Empty))
two-and-half : Optional Rational is Some (Rational (5, 2))
five-fourths : Optional Rational is Some (Rational (5, 4))
five-thirds : Optional Rational is Some (Rational (5, 3))
no-rational : Optional Rational is None Rational

exact-median : Pass is Pass ((median four) = two-and-half)
empty-median : Pass is Pass ((median none) = no-rational)
rational-median : Pass is Pass ((median rational-four) = two-and-half)
stable-modes : Pass is Pass ((modes repeated) = expected-modes)
rational-modes : Pass is Pass ((modes rational-repeated) = expected-rational-modes)
empty-modes : Pass is Pass ((modes none) = none)
histogram-size : Pass is Pass ((entry-count (histogram repeated)) = 3)
rational-histogram-size : Pass is Pass ((entry-count (histogram rational-repeated)) = 3)
empty-histogram : Pass is Pass ((entry-count (histogram none)) = 0)
population-exact : Pass is Pass ((population-variance four) = five-fourths)
rational-population : Pass is Pass ((population-variance rational-four) = five-fourths)
empty-population : Pass is Pass ((population-variance none) = no-rational)
sample-exact : Pass is Pass ((sample-variance four) = five-thirds)
rational-sample : Pass is Pass ((sample-variance rational-four) = five-thirds)
sample-undersized : Pass is Pass ((sample-variance one-value) = no-rational)
middle-quantile : Pass is Pass ((quantile (four, Rational (1, 2))) = two-and-half)
rational-quantile : Pass is Pass ((quantile (rational-four, Rational (1, 2))) = two-and-half)
minimum-quantile : Pass is Pass ((quantile (four, Rational 0)) = (Some (Rational 1)))
maximum-quantile : Pass is Pass ((quantile (four, Rational 1)) = (Some (Rational 4)))
empty-quantile : Pass is Pass ((quantile (none, Rational (1, 2))) = no-rational)
self-covariance : Pass is Pass ((covariance (four, four)) = five-fourths)
rational-covariance : Pass is Pass ((covariance (rational-four, rational-four)) = five-fourths)
empty-covariance : Pass is Pass ((covariance (none, none)) = no-rational)
stream-mean : Pass is Pass ((summary-mean (summarize four)) = two-and-half)
rational-stream-mean : Pass is Pass ((summary-mean (summarize rational-four)) = two-and-half)
stream-variance : Pass is Pass ((summary-population-variance (summarize four)) = five-fourths)
empty-summary : Pass is Pass ((summarize no-rationals) = (0, Rational 0, Rational 0))
empty-summary-mean : Pass is Pass ((summary-mean (summarize no-rationals)) = no-rational)
empty-summary-variance : Pass is Pass ((summary-population-variance (summarize no-rationals)) = no-rational)
extended-summary is summary-add (summarize four, Rational 5)
extended-mean : Pass is Pass ((summary-mean extended-summary) = (Some (Rational 3)))
first-half : List Int is Entry (1, Entry (2, Empty))
second-half : List Int is Entry (3, Entry (4, Empty))
merged-summary is summary-merge (summarize first-half, summarize second-half)
merged-mean : Pass is Pass ((summary-mean merged-summary) = two-and-half)

(exact-median, empty-median, rational-median, stable-modes, rational-modes,
 empty-modes, histogram-size, rational-histogram-size, empty-histogram,
 population-exact, rational-population, empty-population, sample-exact,
 rational-sample, sample-undersized, middle-quantile, rational-quantile,
 minimum-quantile, maximum-quantile, empty-quantile, self-covariance,
 rational-covariance, empty-covariance, stream-mean, rational-stream-mean,
 stream-variance, empty-summary, empty-summary-mean, empty-summary-variance,
 extended-mean, merged-mean)
