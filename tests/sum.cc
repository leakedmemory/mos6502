#include <gtest/gtest.h>

extern "C" {
#include "sum.h"
}

TEST(SumTest, SummingNumbers) { EXPECT_EQ(sum(1, 1), 2); }
