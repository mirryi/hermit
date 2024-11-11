{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE StandaloneDeriving #-}

module SMCDEL.FFI where

import Data.Aeson (FromJSON, ToJSON)
import Foreign.C (CString)
import GHC.Generics (Generic)
import SMCDEL.Language (Agent, Form (..), Prp (..))

data FFIForm
  = FFITop
  | FFIBot
  | FFIPrpF Prp
  | FFINeg FFIForm
  | FFIConj [FFIForm]
  | FFIDisj [FFIForm]
  | FFIXor [FFIForm]
  | FFIImpl FFIForm FFIForm
  | FFIEqui FFIForm FFIForm
  | FFIForall [Prp] FFIForm
  | FFIExists [Prp] FFIForm
  | FFIK Agent FFIForm
  | FFICk [Agent] FFIForm
  | FFIDk [Agent] FFIForm
  | FFIKw Agent FFIForm
  | FFICkw [Agent] FFIForm
  | FFIDkw [Agent] FFIForm
  | FFIPubAnnounce FFIForm FFIForm
  | FFIPubAnnounceW FFIForm FFIForm
  | FFIAnnounce [Agent] FFIForm FFIForm
  | FFIAnnounceW [Agent] FFIForm FFIForm
  deriving (Eq, Ord, Show, Generic)

deriving instance Generic Prp

instance FromJSON Prp

instance ToJSON Prp

instance FromJSON FFIForm

instance ToJSON FFIForm

