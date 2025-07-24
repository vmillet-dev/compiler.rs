int main() {
    int a = 10;
    float b = 3.14;
    char c = 'A';
    
    int result = a + 5;
    float calc = b * 2.0;
    
    println("Values: a=%d, b=%.2f, c=%c", a, b, c);
    println("Results: result=%d, calc=%.2f", result, calc);
    
    return result;
}