# Build Fairy

This project aims to allow replacing the custom made scripts that are present in some builds with a more declarative approach.


# Examples

### Copy

Given an initial structure like the following
```
srcdir/lib1.so
srcdir/folder_a/liba.so
srcdir/doc/my_doc.md
```

With the target structure

```
outdir/lib/lib1.so
outdir/lib/extra/liba.so
outdir/docs/README.md
```

We can accomplish this by declaring a our manifest:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<tasks xmlsns="https://github.com/glecaros/bf">
  <copy>
    <group source="srcdir" destination="outdir">
      <group destination="lib">
        <item>lib1.so</item>
        <group destination="extra">
            <item>folder_a/liba.so</item>
        </group>
      </group>
      <group source="doc" destination="docs">
        <item destination="README.md">my_doc.md</item>
      </group>
    </group>
  </copy>
</tasks>
```

Then, we just run the tool using our manifest as input

```
bf --input input.xml
```