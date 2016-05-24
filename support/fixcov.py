#!/usr/bin/env python2.7

import os, sys

from lxml import objectify, etree
E = objectify.ElementMaker(annotate=False)

def all_rs_sources(path, prefix):
  if prefix[-1] != '/':
    raise RuntimeError('"{}" needs to end with /'.format(path))
  def fixpath(p):
    p = os.path.abspath(p)
    if not p.startswith(prefix):
      raise RuntimeError('expected prefix "{}", got "{}"'.format(prefix, p))
    return p[len(prefix):]
  return [(fixpath(os.path.join(dirpath, f)), os.path.join(os.path.abspath(dirpath), f))
    for dirpath, dirnames, files in os.walk(path)
    for f in files if f.endswith('.rs')]

def source_to_lines(source):
  lines = open(source).readlines()
  out = []
  i = 0
  for l in lines:
    i += 1

    l = l.strip()
    if (
      l.startswith('//')       # comments
      or len(l) == 0           # empty lines
      or l == '}'              # single closing brace
      or l.startswith('use ')  # use
    ):
      continue

    out.append(i)
  return out

def build_source_xml(relpath, source):
  name = os.path.split(source)[-1].replace('.rs', '_rs')
  lines = source_to_lines(source)

  s = '<class name="{}" filename="{}" line-rate="0.000">\n<lines>'.format(name, relpath)
  for l in lines:
    s += '  <line number="{}" hits="0"/>'.format(l)
  s += '</lines></class>'

  return etree.XML(s)

def update_cov(path, cov_file):
  cov = etree.parse(cov_file)
  source_prefix = cov.xpath('//source')[0].text
  existing_files = set(cov.xpath('//class/@filename'))
  all_files = all_rs_sources(path, source_prefix)

  new_files = []
  for f, ap in all_files:
    if not f in existing_files:
      new_files.append((f, ap))

  classes = cov.xpath('//classes')[0]
  for f, ap in new_files:
    classes.append(build_source_xml(f, ap))

  genxml = etree.tostring(cov)
  f = open(cov_file, 'w')
  f.write(genxml)
  f.close()
  print " * fixcov added {} empty coverge files to {}".format(len(new_files), cov_file)

if __name__ == '__main__':
  update_cov(sys.argv[1], sys.argv[2])
