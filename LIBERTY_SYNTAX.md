# Liberty Syntax

## Unexplored syntax

- Group names can have bus syntax

    pin (x[0:3]) { ... }

- Group names can be a list of names

    timing (B0_X0, B0_x1, ... ) { .. }

- Some complex attributes can be 1D, 2D or 3D lists of float values

    index_1( "float, ..., float");
    values ( "1.0, 2.0, 3.0" );

    index_1( "float, ..., float_m");
    index_2( "float, ..., float_n");
    values ( \
             "float, ..., float_n",
             ...
             "float, ..., float_n", /* float_m rows in group */
    );
                

    index_1( "float, ..., float_m");
    index_2( "float, ..., float_n");
    index_3( "float, ..., float_o");
    values ( \
             "float, ..., float_o",
             ...
             "float, ..., float_o", /* float_n rows in group  */

             ...

             "float, ..., float_o",
             ...
             "float, ..., float_o", /* float_n groups of groups */
    );

  Could this be opportunistically parsed? Each quotes may be parsed as a sequence
  of floating point values, else it's a normal string.

