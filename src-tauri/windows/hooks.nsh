; Pass the language chosen in the installer to the app's first launch.
; 1055 = Turkish, anything else falls back to English.

!macro NSIS_HOOK_POSTINSTALL
  ${If} $LANGUAGE == 1055
    WriteRegStr HKCU "Software\Foxivo\Sunucu Analiz" "Language" "tr"
  ${Else}
    WriteRegStr HKCU "Software\Foxivo\Sunucu Analiz" "Language" "en"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey HKCU "Software\Foxivo\Sunucu Analiz"
!macroend
