// Test dictionaries.

---
// Empty
{(:)}

// Two pairs.
{(a1: 1, a2: 2)}

---
// Simple expression after already being identified as a dictionary.
// Error: 9-10 expected named pair, found expression
{(a: 1, b)}

// Identified as dictionary due to initial colon.
// Error: 4:4-4:5 expected named pair, found expression
// Error: 3:5 expected comma
// Error: 2:12-2:16 expected identifier
// Error: 1:17-1:18 expected expression, found colon
{(:1 b:"", true::)}
