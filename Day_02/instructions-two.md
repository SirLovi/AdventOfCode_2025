## \--- Part Two ---

The clerk quickly discovers that there are still invalid IDs in the ranges in your list. Maybe the young Elf was doing other silly patterns as well?

Now, an ID is invalid if it is made only of some sequence of digits repeated _at least_ twice. So, `12341234` (`1234` two times), `123123123` (`123` three times), `1212121212` (`12` five times), and `1111111` (`1` seven times) are all invalid IDs.

From the same example as before:

  * `11-22` still has two invalid IDs, `_11_` and `_22_`.
  * `95-115` now has two invalid IDs, `_99_` and `_111_`.
  * `998-1012` now has two invalid IDs, `_999_` and `_1010_`.
  * `1188511880-1188511890` still has one invalid ID, `_1188511885_`.
  * `222220-222224` still has one invalid ID, `_222222_`.
  * `1698522-1698528` still contains no invalid IDs.
  * `446443-446449` still has one invalid ID, `_446446_`.
  * `38593856-38593862` still has one invalid ID, `_38593859_`.
  * `565653-565659` now has one invalid ID, `_565656_`.
  * `824824821-824824827` now has one invalid ID, `_824824824_`.
  * `2121212118-2121212124` now has one invalid ID, `_2121212121_`.



Adding up all the invalid IDs in this example produces `_4174379265_`.

_What do you get if you add up all of the invalid IDs using these new rules?_