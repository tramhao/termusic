## vim:tw=78
## Process this file with automake to create Makefile.in
##
## This file is part of SoundTouch, an audio processing library for pitch/time adjustments
## 
## SoundTouch is free software; you can redistribute it and/or modify it under the
## terms of the GNU General Public License as published by the Free Software
## Foundation; either version 2 of the License, or (at your option) any later
## version.
## 
## SoundTouch is distributed in the hope that it will be useful, but WITHOUT ANY
## WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
## A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
## 
## You should have received a copy of the GNU General Public License along with
## this program; if not, write to the Free Software Foundation, Inc., 59 Temple
## Place - Suite 330, Boston, MA  02111-1307, USA

## These are common definitions used in all Makefiles
## It is actually included when a makefile.am is converted to Makefile.in
## by automake, so it's ok to have @MACROS@ that will be set by configure

AM_CPPFLAGS=-I$(top_srcdir)/include

# doc directory
pkgdocdir=$(prefix)/doc/@PACKAGE@
