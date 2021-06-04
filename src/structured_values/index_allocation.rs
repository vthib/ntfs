// Copyright 2021 Colin Finck <colin@reactos.org>
// SPDX-License-Identifier: GPL-2.0-or-later

use crate::attribute_value::NtfsAttributeValue;
use crate::error::Result;
use crate::index_record::NtfsIndexRecord;
use crate::structured_values::index_root::NtfsIndexRoot;
use crate::structured_values::NewNtfsStructuredValue;
use crate::traits::NtfsReadSeek;
use binread::io::{Read, Seek, SeekFrom};
use core::iter::FusedIterator;

#[derive(Clone, Debug)]
pub struct NtfsIndexAllocation<'n> {
    value: NtfsAttributeValue<'n>,
}

impl<'n> NtfsIndexAllocation<'n> {
    pub fn iter(&self, index_root: &NtfsIndexRoot) -> NtfsIndexRecords<'n> {
        let index_record_size = index_root.index_record_size();
        NtfsIndexRecords::new(self.value.clone(), index_record_size)
    }
}

impl<'n> NewNtfsStructuredValue<'n> for NtfsIndexAllocation<'n> {
    fn new<T>(_fs: &mut T, value: NtfsAttributeValue<'n>, _length: u64) -> Result<Self>
    where
        T: Read + Seek,
    {
        Ok(Self { value })
    }
}

#[derive(Clone, Debug)]
pub struct NtfsIndexRecords<'n> {
    value: NtfsAttributeValue<'n>,
    index_record_size: u32,
}

impl<'n> NtfsIndexRecords<'n> {
    fn new(value: NtfsAttributeValue<'n>, index_record_size: u32) -> Self {
        Self {
            value,
            index_record_size,
        }
    }

    pub fn attach<'a, T>(self, fs: &'a mut T) -> NtfsIndexRecordsAttached<'n, 'a, T>
    where
        T: Read + Seek,
    {
        NtfsIndexRecordsAttached::new(fs, self)
    }

    pub fn next<T>(&mut self, fs: &mut T) -> Option<Result<NtfsIndexRecord<'n>>>
    where
        T: Read + Seek,
    {
        if self.value.stream_position() >= self.value.len() {
            return None;
        }

        // Get the current record.
        let record = iter_try!(NtfsIndexRecord::new(
            fs,
            self.value.clone(),
            self.index_record_size
        ));

        // Advance our iterator to the next record.
        iter_try!(self
            .value
            .seek(fs, SeekFrom::Current(self.index_record_size as i64)));

        Some(Ok(record))
    }
}

pub struct NtfsIndexRecordsAttached<'n, 'a, T>
where
    T: Read + Seek,
{
    fs: &'a mut T,
    index_records: NtfsIndexRecords<'n>,
}

impl<'n, 'a, T> NtfsIndexRecordsAttached<'n, 'a, T>
where
    T: Read + Seek,
{
    fn new(fs: &'a mut T, index_records: NtfsIndexRecords<'n>) -> Self {
        Self { fs, index_records }
    }

    pub fn detach(self) -> NtfsIndexRecords<'n> {
        self.index_records
    }
}

impl<'n, 'a, T> Iterator for NtfsIndexRecordsAttached<'n, 'a, T>
where
    T: Read + Seek,
{
    type Item = Result<NtfsIndexRecord<'n>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index_records.next(self.fs)
    }
}

impl<'n, 'a, T> FusedIterator for NtfsIndexRecordsAttached<'n, 'a, T> where T: Read + Seek {}