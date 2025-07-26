int factorial(int n) {
    println("toto");
    println("factorial n param: %d", n);
    int result = 1;
    for (int i = 1; i <= n; i = i + 1) {
        result = result * i;
    }
    return result;
}

int main() {
    int x = 5;
    int y = 0;

    int logical_and = x && y;
    int logical_or = x || y;
    int complex_logic = (x > 3) && (y == 0);

    int count = 0;
    int i = 0;
    while (i < 10) {
        i = i + 1;
        if (i == 3) {
            continue;
        }
        if (i == 8) {
            break;
        }
        count = count + 1;
    }
    int fact_result = factorial(4);

    println("count = %d, fact_result = %d", count, fact_result);
    return 0;
}