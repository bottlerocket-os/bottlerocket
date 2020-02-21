#!/bin/bash

# Docker uses MCS labels for container separation. It picks low and
# high numbers from the range of 1024 categories to create a unique
# label that does not dominate and is not dominated by other labels.
# For this to work, all categories must be included in the policy.

for i in {0..1023} ; do
  echo "(category c${i})"
done

echo "(categoryorder ("
for i in {0..1023} ; do
  echo -n " c${i}"
done
echo "))"
