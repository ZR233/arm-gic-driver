use core::{
    fmt::{self, Debug, Formatter},
    ops::Range,
};

#[derive(Clone, Copy)]
pub enum SGITarget<'a> {
    AllOther,
    Targets(&'a [CPUTarget]),
}

#[derive(Clone, Copy)]
pub struct CPUTarget {
    pub aff0: u8,
    pub aff1: u8,
    pub aff2: u8,
    pub aff3: u8,
}

impl From<MPID> for CPUTarget {
    fn from(value: MPID) -> Self {
        Self {
            aff0: value.aff0,
            aff1: value.aff1,
            aff2: value.aff2,
            aff3: value.aff3 as _,
        }
    }
}
impl CPUTarget {
    pub const CORE0: CPUTarget = CPUTarget {
        aff0: 0,
        aff1: 0,
        aff2: 0,
        aff3: 0,
    };

    pub(crate) fn affinity(&self) -> u32 {
        self.aff0 as u32
            | (self.aff1 as u32) << 8
            | (self.aff2 as u32) << 16
            | (self.aff3 as u32) << 24
    }

    pub(crate) fn cpu_target_list(&self) -> u8 {
        1 << self.aff0
    }
}

#[repr(C)]
pub struct MPID {
    pub aff0: u8,
    pub aff1: u8,
    pub aff2: u8,
    _flag: u8,
    pub aff3: u32,
}

impl From<u64> for MPID {
    fn from(value: u64) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

impl From<usize> for MPID {
    fn from(value: usize) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

/// Interrupt ID 0-15 are used for SGIs (Software-generated interrupt).
///
/// SGI is an interrupt generated by software writing to a GICD_SGIR register in
/// the GIC. The system uses SGIs for interprocessor communication.
pub const SGI_RANGE: Range<u32> = Range { start: 0, end: 16 };

/// Interrupt ID 16-31 are used for PPIs (Private Peripheral Interrupt).
///
/// PPI is a peripheral interrupt that is specific to a single processor.
pub const PPI_RANGE: Range<u32> = Range { start: 16, end: 32 };

/// Interrupt ID 32-1019 are used for SPIs (Shared Peripheral Interrupt).
///
/// SPI is a peripheral interrupt that the Distributor can route to any of a
/// specified combination of processors.
pub const SPI_RANGE: Range<u32> = Range {
    start: 32,
    end: 1020,
};

pub const SPECIAL_RANGE: Range<u32> = Range {
    start: 1020,
    end: 1024,
};

/// An interrupt ID.
#[derive(Copy, Clone, Eq, Ord, PartialOrd, PartialEq)]
pub struct IntId(u32);

impl IntId {
    /// Create a new `IntId` from a raw ID.
    /// # Safety
    /// `id` must be transformed into a valid [IntId]
    pub const unsafe fn raw(id: u32) -> Self {
        assert!(id < SPECIAL_RANGE.end);
        Self(id)
    }

    /// Returns the interrupt ID for the given Software Generated Interrupt.
    pub const fn sgi(sgi: u32) -> Self {
        assert!(sgi < SGI_RANGE.end);
        Self(sgi)
    }

    /// Returns the interrupt ID for the given Private Peripheral Interrupt.
    pub const fn ppi(ppi: u32) -> Self {
        assert!(ppi < PPI_RANGE.end - PPI_RANGE.start);
        Self(PPI_RANGE.start + ppi)
    }

    /// Returns the interrupt ID for the given Shared Peripheral Interrupt.
    pub const fn spi(spi: u32) -> Self {
        assert!(spi < SPECIAL_RANGE.start);
        Self(SPI_RANGE.start + spi)
    }

    /// Returns whether this interrupt ID is for a Software Generated Interrupt.
    pub fn is_sgi(&self) -> bool {
        SGI_RANGE.contains(&self.0)
    }

    /// Returns whether this interrupt ID is private to a core, i.e. it is an SGI or PPI.
    pub fn is_private(&self) -> bool {
        self.0 < SPI_RANGE.start
    }

    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

impl Debug for IntId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.0 {
            0..16 => write!(f, "SGI {}", self.0 - SGI_RANGE.start),
            16..32 => write!(f, "PPI {}", self.0 - PPI_RANGE.start),
            32..1020 => write!(f, "SPI {}", self.0 - SPI_RANGE.start),
            1020..1024 => write!(f, "Special IntId{}", self.0),
            _ => write!(f, "Invalid IntId{}", self.0),
        }
    }
}

impl From<IntId> for u32 {
    fn from(intid: IntId) -> Self {
        intid.0
    }
}

/// The trigger configuration for an interrupt.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Trigger {
    /// The interrupt is edge triggered.
    Edge,
    /// The interrupt is level triggered.
    Level,
}

#[derive(Debug, Clone)]
pub enum GicError {
    Notimplemented,
    Timeout,
}

pub type GicResult<T = ()> = core::result::Result<T, GicError>;

pub trait GicGeneric {
    fn get_and_acknowledge_interrupt(&self) -> Option<IntId>;
    fn end_interrupt(&self, intid: IntId);
    fn irq_max_size(&self) -> usize;
    fn irq_enable(&mut self, intid: IntId);
    fn irq_disable(&mut self, intid: IntId);
    fn set_priority(&mut self, intid: IntId, priority: usize);
    fn set_trigger(&mut self, intid: IntId, trigger: Trigger);
    fn set_bind_cpu(&mut self, intid: IntId, cpu_list: &[CPUTarget]);
    fn current_cpu_setup(&self);
}
