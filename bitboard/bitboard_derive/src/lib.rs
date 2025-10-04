use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Bitboard)]
pub fn bitboard_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let implementation = quote! {
        impl BitboardTrait for #name {
            #[inline]
            fn new() -> Self {
                Self { cols: [0; 10] }
            }
            
            #[inline(always)]
            fn fold_and(&self) -> u64 {
                self.cols.iter().fold(!0, |a, &b| a & b)
            }

            #[inline(always)]
            fn fold_or(&self) -> u64 {
                self.cols.iter().fold(0, |a, &b| a | b)
            }

            #[inline(always)]
            fn fold_xor(&self) -> u64 {
                self.cols.iter().fold(0, |a, &b| a ^ b)
            }
        }

        impl std::ops::Index<usize> for #name {
            type Output = u64;

            fn index(&self, idx: usize) -> &Self::Output {
                &self.cols[idx]
            }
        }

        impl std::ops::IndexMut<usize> for #name {
            fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
                &mut self.cols[idx]
            }
        }

        // Implementation for the standard Not trait
        impl std::ops::Not for #name {
            type Output = Self;

            #[inline(always)]
            fn not(self) -> Self::Output {
                Self { cols: self.cols.map(|x| !x) }
            }
        }

        impl std::ops::BitXor for #name {
            type Output = Self;

            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self { cols: std::array::from_fn(|x| self.cols[x] ^ rhs.cols[x] ) }
            }
        }

        impl std::ops::BitOr for #name {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self { cols: std::array::from_fn(|x| self.cols[x] | rhs.cols[x] ) }
            }
        }

        impl std::ops::BitAnd for #name {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self { cols: std::array::from_fn(|x| self.cols[x] & rhs.cols[x] ) }
            }
        }
    };
    implementation.into()
}