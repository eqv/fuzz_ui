#!/bin/sh -e

# called by uscan with '--upstream-version' <version> <file>

tar xzf $3
rm -rf dmg2img*/debian
rm -rf dmg2img*/*.spec
tar czf $3 dmg2img*
rm -rf dmg2img-*

# move to directory 'tarballs'
if [ -r .svn/deb-layout ]; then
  . .svn/deb-layout
  mv $3 $origDir
  echo "moved $3 to $origDir"
fi

exit 0

