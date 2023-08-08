import { JSON } from "assemblyscript-json/assembly";
import * as reso from "./reso";

export function validate(): void {
  const data = reso.data();
  const previousData = reso.previousData();

  if (!data.isObj) {
    reso.diagnostic("Data was not an object");
    unreachable();
  }

  const obj = <JSON.Obj>data;

  const listPriceOrNull = obj.getFloat("ListPrice");
  let listPrice = 0.0;
  if (listPriceOrNull !== null) {
    listPrice = listPriceOrNull.valueOf();
  }

  const mlsStatusOrNull = obj.getString("MlsStatus");
  let mlsStatus: string | null = null;
  if (mlsStatusOrNull !== null) {
    mlsStatus = mlsStatusOrNull.valueOf();
  }

  if (listPrice <= 0.0) {
    reso.error("ListPrice", "List price must be greater than $0");
  }

  if (mlsStatus === "Closed") {
    reso.setRequired("ClosePrice", true);
    reso.setDisplay("ClosePrice", true);
  } else {
    reso.setRequired("ClosePrice", false);
    reso.setDisplay("ClosePrice", false);
    reso.set("ClosePrice", new JSON.Null());
  }
}
